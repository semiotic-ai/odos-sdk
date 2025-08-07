use std::time::Duration;

use backoff::{backoff::Backoff, ExponentialBackoff};
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use tokio::time::timeout;
use tracing::{debug, instrument, warn};

use crate::error::{OdosError, Result};

/// Configuration for the HTTP client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Request timeout duration
    pub timeout: Duration,
    /// Connection timeout duration
    pub connect_timeout: Duration,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial retry delay
    pub initial_retry_delay: Duration,
    /// Maximum retry delay
    pub max_retry_delay: Duration,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Connection pool idle timeout
    pub pool_idle_timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_retries: 3,
            initial_retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(5),
            max_connections: 20,
            pool_idle_timeout: Duration::from_secs(90),
        }
    }
}

/// Enhanced HTTP client with retry logic and timeouts
#[derive(Debug, Clone)]
pub struct OdosHttpClient {
    client: Client,
    config: ClientConfig,
}

impl OdosHttpClient {
    /// Create a new HTTP client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ClientConfig::default())
    }

    /// Create a new HTTP client with custom configuration
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .pool_max_idle_per_host(config.max_connections)
            .pool_idle_timeout(config.pool_idle_timeout)
            .build()
            .map_err(OdosError::Http)?;

        Ok(Self { client, config })
    }

    /// Execute a request with retry logic
    #[instrument(skip(self, request_builder_fn), level = "debug")]
    pub async fn execute_with_retry<F>(&self, request_builder_fn: F) -> Result<Response>
    where
        F: Fn() -> RequestBuilder + Clone,
    {
        let mut backoff = ExponentialBackoff {
            initial_interval: self.config.initial_retry_delay,
            max_interval: self.config.max_retry_delay,
            max_elapsed_time: Some(self.config.timeout),
            ..Default::default()
        };

        let mut attempt = 0;

        loop {
            attempt += 1;
            debug!(attempt, "Executing HTTP request");

            let request = match request_builder_fn().build() {
                Ok(req) => req,
                Err(e) => return Err(OdosError::Http(e)),
            };

            let last_error = match timeout(self.config.timeout, self.client.execute(request)).await
            {
                Ok(Ok(response)) if response.status().is_success() => {
                    debug!(attempt, status = %response.status(), "Request successful");
                    return Ok(response);
                }
                Ok(Ok(response)) => {
                    let status = response.status();

                    if status == StatusCode::TOO_MANY_REQUESTS {
                        let retry_after = extract_retry_after(&response);

                        let body = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());

                        let error = OdosError::rate_limit_error(body);

                        if !error.is_retryable() {
                            return Err(error);
                        }

                        warn!(
                            attempt,
                            status = %status,
                            retry_after_secs = ?retry_after.map(|d| d.as_secs()),
                            "Rate limit exceeded (429), will retry after delay"
                        );

                        if let Some(delay) = retry_after {
                            debug!(?delay, "Respecting Retry-After header");
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                        error
                    } else {
                        let body = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());

                        let error = OdosError::api_error(status, body);

                        if !error.is_retryable() {
                            return Err(error);
                        }

                        debug!(attempt, status = %status, "Retryable API error, retrying");
                        error
                    }
                }
                Ok(Err(e)) => {
                    let error = OdosError::Http(e);

                    if !error.is_retryable() {
                        return Err(error);
                    }
                    warn!(attempt, error = %error, "Retryable HTTP error, retrying");
                    error
                }
                Err(_) => {
                    let error = OdosError::timeout_error("Request timed out");
                    warn!(attempt, timeout = ?self.config.timeout, "Request timed out, retrying");
                    error
                }
            };

            // Check if we've exhausted retries
            if attempt >= self.config.max_retries {
                return Err(last_error);
            }

            if let Some(delay) = backoff.next_backoff() {
                debug!(?delay, "Waiting before retry");
                tokio::time::sleep(delay).await;
            } else {
                return Err(last_error);
            }
        }
    }

    /// Get a reference to the underlying reqwest client
    pub fn inner(&self) -> &Client {
        &self.client
    }

    /// Get the client configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }
}

/// Extract the retry-after header from the response
fn extract_retry_after(response: &Response) -> Option<Duration> {
    response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
}

impl Default for OdosHttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, Request, ResponseTemplate,
    };

    /// Helper to create a mock that returns different responses based on attempt count
    fn create_retry_mock(
        first_status: u16,
        first_body: String,
        success_after: usize,
    ) -> impl Fn(&Request) -> ResponseTemplate {
        let attempt_count = Arc::new(Mutex::new(0));
        move |_req: &Request| {
            let mut count = attempt_count.lock().unwrap();
            *count += 1;

            if *count < success_after {
                ResponseTemplate::new(first_status).set_body_string(&first_body)
            } else {
                ResponseTemplate::new(200).set_body_string("Success")
            }
        }
    }

    /// Helper to create a mock with retry-after header
    fn create_rate_limit_mock(
        retry_after_secs: Option<u64>,
    ) -> impl Fn(&Request) -> ResponseTemplate {
        let attempt_count = Arc::new(Mutex::new(0));
        move |_req: &Request| {
            let mut count = attempt_count.lock().unwrap();
            *count += 1;

            if *count == 1 {
                let mut response =
                    ResponseTemplate::new(429).set_body_string("Rate limit exceeded");
                if let Some(secs) = retry_after_secs {
                    response = response.insert_header("retry-after", secs.to_string());
                }
                response
            } else {
                ResponseTemplate::new(200).set_body_string("Success")
            }
        }
    }

    /// Helper to create a test client with custom config
    fn create_test_client(max_retries: u32, timeout_ms: u64) -> OdosHttpClient {
        let config = ClientConfig {
            max_retries,
            timeout: Duration::from_millis(timeout_ms),
            initial_retry_delay: Duration::from_millis(10),
            max_retry_delay: Duration::from_millis(50),
            ..Default::default()
        };
        OdosHttpClient::with_config(config).unwrap()
    }

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.max_connections, 20);
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = OdosHttpClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_client_with_custom_config() {
        let config = ClientConfig {
            timeout: Duration::from_secs(60),
            max_retries: 5,
            ..Default::default()
        };
        let client = OdosHttpClient::with_config(config.clone());
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.config().timeout, Duration::from_secs(60));
        assert_eq!(client.config().max_retries, 5);
    }

    #[tokio::test]
    async fn test_rate_limit_with_retry_after() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(create_rate_limit_mock(Some(1)))
            .mount(&mock_server)
            .await;

        let client = create_test_client(3, 30000);
        let start = std::time::Instant::now();

        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        assert!(
            response.is_ok(),
            "Request should succeed after retry, but got: {response:?}"
        );

        // Should have waited at least 1 second (from Retry-After header)
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(900),
            "Should respect Retry-After header, elapsed: {elapsed:?}"
        );
    }

    #[tokio::test]
    async fn test_rate_limit_without_retry_after() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(create_rate_limit_mock(None))
            .mount(&mock_server)
            .await;

        let client = create_test_client(3, 30000);
        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        assert!(
            response.is_ok(),
            "Request should succeed after retry, but got: {response:?}"
        );
    }

    #[tokio::test]
    async fn test_non_retryable_error() {
        let mock_server = MockServer::start().await;

        // Returns 400 Bad Request (non-retryable)
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad request"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            max_retries: 3,
            ..Default::default()
        };
        let client = OdosHttpClient::with_config(config).unwrap();

        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        // Should fail immediately without retrying
        assert!(response.is_err());
        if let Err(e) = response {
            assert!(!e.is_retryable());
        }
    }

    #[tokio::test]
    async fn test_retry_exhaustion_returns_last_error() {
        let mock_server = MockServer::start().await;

        // Always returns 503 Service Unavailable (retryable)
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service unavailable"))
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            max_retries: 2,
            initial_retry_delay: Duration::from_millis(10),
            max_retry_delay: Duration::from_millis(50),
            ..Default::default()
        };
        let client = OdosHttpClient::with_config(config).unwrap();

        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        // Should fail after exhausting retries
        assert!(response.is_err());
        if let Err(e) = response {
            assert!(
                matches!(e, OdosError::Api { status, .. } if status == StatusCode::SERVICE_UNAVAILABLE)
            );
        }
    }

    #[tokio::test]
    async fn test_timeout_error() {
        let mock_server = MockServer::start().await;

        // Delays response longer than timeout
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("Success")
                    .set_delay(Duration::from_secs(5)),
            )
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            timeout: Duration::from_millis(100),
            max_retries: 2,
            initial_retry_delay: Duration::from_millis(10),
            ..Default::default()
        };
        let client = OdosHttpClient::with_config(config).unwrap();

        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        // Should fail with timeout error (could be either Http timeout or our Timeout wrapper)
        assert!(response.is_err());
        if let Err(e) = response {
            // Accept either OdosError::Http with timeout or OdosError::Timeout
            let is_timeout = matches!(e, OdosError::Timeout(_))
                || matches!(e, OdosError::Http(ref err) if err.is_timeout());
            assert!(is_timeout, "Expected timeout error, got: {e:?}");
        }
    }

    #[tokio::test]
    async fn test_invalid_request_builder_fails_immediately() {
        let client = OdosHttpClient::default();

        // Create a request builder that will fail on .build()
        // Use an absurdly long header name that will fail validation
        let bad_builder = || {
            let mut builder = client.inner().get("http://localhost");
            // Add an invalid header that will cause build to fail
            builder = builder.header("x".repeat(100000), "value");
            builder
        };

        let result = client.execute_with_retry(bad_builder).await;

        // Should fail immediately without retrying
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, OdosError::Http(_)));
        }
    }

    #[tokio::test]
    async fn test_retryable_500_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(create_retry_mock(
                500,
                "Internal server error".to_string(),
                2,
            ))
            .mount(&mock_server)
            .await;

        let client = create_test_client(3, 30000);
        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        assert!(response.is_ok(), "500 error should be retried and succeed");
    }

    #[tokio::test]
    async fn test_retryable_502_bad_gateway() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(create_retry_mock(502, "Bad gateway".to_string(), 2))
            .mount(&mock_server)
            .await;

        let client = create_test_client(3, 30000);
        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        assert!(response.is_ok(), "502 error should be retried and succeed");
    }

    #[tokio::test]
    async fn test_retryable_503_service_unavailable() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(create_retry_mock(503, "Service unavailable".to_string(), 3))
            .mount(&mock_server)
            .await;

        let client = create_test_client(3, 30000);
        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        assert!(response.is_ok(), "503 error should be retried and succeed");
    }

    #[tokio::test]
    async fn test_retryable_504_gateway_timeout() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(create_retry_mock(504, "Gateway timeout".to_string(), 2))
            .mount(&mock_server)
            .await;

        let client = create_test_client(3, 30000);
        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        assert!(response.is_ok(), "504 error should be retried and succeed");
    }

    #[tokio::test]
    async fn test_network_error_retryable() {
        // Test with an invalid URL that will cause a connection error
        let config = ClientConfig {
            max_retries: 2,
            initial_retry_delay: Duration::from_millis(10),
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let client = OdosHttpClient::with_config(config).unwrap();

        let response = client
            .execute_with_retry(|| client.inner().get("http://localhost:1"))
            .await;

        // Should fail after retries
        assert!(response.is_err());
        if let Err(e) = response {
            assert!(matches!(e, OdosError::Http(_)));
        }
    }

    #[test]
    fn test_accessor_methods() {
        let config = ClientConfig {
            timeout: Duration::from_secs(45),
            max_retries: 5,
            ..Default::default()
        };
        let client = OdosHttpClient::with_config(config.clone()).unwrap();

        // Test config() accessor
        assert_eq!(client.config().timeout, Duration::from_secs(45));
        assert_eq!(client.config().max_retries, 5);

        // Test inner() accessor - just verify it returns a Client
        let _inner: &reqwest::Client = client.inner();
    }

    #[test]
    fn test_default_client() {
        let client = OdosHttpClient::default();

        // Should use default config
        assert_eq!(client.config().timeout, Duration::from_secs(30));
        assert_eq!(client.config().max_retries, 3);
    }

    #[test]
    fn test_extract_retry_after_valid_numeric() {
        let response = reqwest::Response::from(
            http::Response::builder()
                .status(429)
                .header("retry-after", "30")
                .body("")
                .unwrap(),
        );

        let retry_after = extract_retry_after(&response);
        assert_eq!(retry_after, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_extract_retry_after_missing_header() {
        let response =
            reqwest::Response::from(http::Response::builder().status(429).body("").unwrap());

        let retry_after = extract_retry_after(&response);
        assert_eq!(retry_after, None);
    }

    #[test]
    fn test_extract_retry_after_malformed_value() {
        let response = reqwest::Response::from(
            http::Response::builder()
                .status(429)
                .header("retry-after", "not-a-number")
                .body("")
                .unwrap(),
        );

        let retry_after = extract_retry_after(&response);
        assert_eq!(retry_after, None);
    }

    #[test]
    fn test_extract_retry_after_zero_value() {
        let response = reqwest::Response::from(
            http::Response::builder()
                .status(429)
                .header("retry-after", "0")
                .body("")
                .unwrap(),
        );

        let retry_after = extract_retry_after(&response);
        assert_eq!(retry_after, Some(Duration::from_secs(0)));
    }

    #[test]
    fn test_extract_retry_after_large_value() {
        let response = reqwest::Response::from(
            http::Response::builder()
                .status(429)
                .header("retry-after", "3600")
                .body("")
                .unwrap(),
        );

        let retry_after = extract_retry_after(&response);
        assert_eq!(retry_after, Some(Duration::from_secs(3600)));
    }

    #[test]
    fn test_extract_retry_after_invalid_utf8() {
        let response = reqwest::Response::from(
            http::Response::builder()
                .status(429)
                .header("retry-after", vec![0xff, 0xfe])
                .body("")
                .unwrap(),
        );

        let retry_after = extract_retry_after(&response);
        assert_eq!(retry_after, None);
    }

    #[tokio::test]
    async fn test_max_retries_zero() {
        let mock_server = MockServer::start().await;

        // Mock that would normally trigger retries
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Server error"))
            .expect(1) // Should only be called once
            .mount(&mock_server)
            .await;

        let client = create_test_client(0, 30000); // max_retries = 0
        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        // Should fail immediately without retrying
        assert!(response.is_err());
        if let Err(e) = response {
            assert!(
                matches!(e, OdosError::Api { status, .. } if status == StatusCode::INTERNAL_SERVER_ERROR)
            );
        }
    }

    #[tokio::test]
    async fn test_client_config_failure() {
        // Test creating a client with an invalid configuration
        // Using an extremely high connection limit that might cause issues
        let config = ClientConfig {
            max_connections: usize::MAX,
            ..Default::default()
        };

        // This might not actually fail with reqwest, but we test the error handling path
        let result = OdosHttpClient::with_config(config);

        // If it succeeds, that's fine - reqwest is quite permissive
        // If it fails, we verify proper error wrapping
        match result {
            Ok(_) => {
                // Client creation succeeded - this is actually normal
            }
            Err(e) => {
                // If it fails, should be wrapped as Http error
                assert!(matches!(e, OdosError::Http(_)));
            }
        }
    }
}
