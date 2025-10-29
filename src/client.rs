use std::time::Duration;

use backoff::{backoff::Backoff, ExponentialBackoff};
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use tokio::time::timeout;
use tracing::{debug, instrument};

use crate::{
    api::OdosApiErrorResponse,
    error::{OdosError, Result},
    error_code::OdosErrorCode,
};

/// Configuration for retry behavior
///
/// Controls which errors should be retried and how retries are executed.
///
/// # Examples
///
/// ```rust
/// use odos_sdk::RetryConfig;
///
/// // No retries - all errors return immediately
/// let config = RetryConfig::no_retries();
///
/// // Conservative retries - only network errors
/// let config = RetryConfig::conservative();
///
/// // Default retries - network errors and server errors
/// let config = RetryConfig::default();
///
/// // Custom retry logic
/// let config = RetryConfig {
///     max_retries: 2,
///     retry_server_errors: false,
///     retry_predicate: Some(|err| {
///         // Custom logic to determine if error should be retried
///         err.is_retryable()
///     }),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum retry attempts for retryable errors
    pub max_retries: u32,

    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,

    /// Whether to retry server errors (5xx)
    pub retry_server_errors: bool,

    /// Custom retry predicate (advanced use)
    ///
    /// When provided, this function overrides the default retry logic.
    /// Return `true` to retry the error, `false` to return it immediately.
    pub retry_predicate: Option<fn(&OdosError) -> bool>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            retry_server_errors: true,
            retry_predicate: None,
        }
    }
}

impl RetryConfig {
    /// No retries - return errors immediately
    ///
    /// Use this when you want to handle all errors at the application level,
    /// or when implementing your own retry logic.
    pub fn no_retries() -> Self {
        Self {
            max_retries: 0,
            ..Default::default()
        }
    }

    /// Conservative retries - only network errors
    ///
    /// This configuration retries only transient network failures
    /// (timeouts, connection errors) but not server errors (5xx).
    /// Use this when you want to be cautious about retry behavior.
    pub fn conservative() -> Self {
        Self {
            max_retries: 2,
            retry_server_errors: false,
            ..Default::default()
        }
    }
}

/// Configuration for the HTTP client
///
/// Combines connection settings with retry behavior configuration.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Request timeout duration
    pub timeout: Duration,
    /// Connection timeout duration
    pub connect_timeout: Duration,
    /// Retry behavior configuration
    pub retry_config: RetryConfig,
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
            retry_config: RetryConfig::default(),
            max_connections: 20,
            pool_idle_timeout: Duration::from_secs(90),
        }
    }
}

impl ClientConfig {
    /// Create a configuration with no retries
    ///
    /// Useful when you want to handle all errors at the application level.
    pub fn no_retries() -> Self {
        Self {
            retry_config: RetryConfig::no_retries(),
            ..Default::default()
        }
    }

    /// Create a configuration with conservative retry behavior
    ///
    /// Only retries transient network failures, not server errors or rate limits.
    pub fn conservative() -> Self {
        Self {
            retry_config: RetryConfig::conservative(),
            ..Default::default()
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
        let initial_backoff_duration =
            Duration::from_millis(self.config.retry_config.initial_backoff_ms);
        let mut backoff = ExponentialBackoff {
            initial_interval: initial_backoff_duration,
            max_interval: Duration::from_secs(30), // Max backoff of 30 seconds
            max_elapsed_time: Some(self.config.timeout),
            ..Default::default()
        };

        let mut attempt = 0;

        loop {
            attempt += 1;

            let request = match request_builder_fn().build() {
                Ok(req) => req,
                Err(e) => return Err(OdosError::Http(e)),
            };

            let last_error = match timeout(self.config.timeout, self.client.execute(request)).await
            {
                Ok(Ok(response)) if response.status().is_success() => {
                    return Ok(response);
                }
                Ok(Ok(response)) => {
                    let status = response.status();

                    if status == StatusCode::TOO_MANY_REQUESTS {
                        let retry_after = extract_retry_after(&response);

                        // Parse structured error response
                        let (message, _code, _trace_id) = parse_error_response(response).await;

                        let error =
                            OdosError::rate_limit_error_with_retry_after(message, retry_after);

                        // Rate limits are never retried - return immediately
                        if !self.should_retry(&error, attempt) {
                            return Err(error);
                        }

                        if let Some(delay) = retry_after {
                            // If retry-after is 0, use exponential backoff instead
                            if !delay.is_zero() {
                                debug!(attempt, retry_after_secs = delay.as_secs());
                                tokio::time::sleep(delay).await;
                                continue;
                            }
                        }
                        error
                    } else {
                        // Parse structured error response
                        let (message, code, trace_id) = parse_error_response(response).await;

                        let error = OdosError::api_error_with_code(status, message, code, trace_id);

                        if !self.should_retry(&error, attempt) {
                            return Err(error);
                        }

                        error
                    }
                }
                Ok(Err(e)) => {
                    let error = OdosError::Http(e);

                    if !self.should_retry(&error, attempt) {
                        return Err(error);
                    }
                    debug!(attempt, error = %error);
                    error
                }
                Err(_) => {
                    let error = OdosError::timeout_error("Request timed out");
                    debug!(attempt, timeout = ?self.config.timeout);
                    error
                }
            };

            // Check if we've exhausted retries
            if attempt >= self.config.retry_config.max_retries {
                return Err(last_error);
            }

            if let Some(delay) = backoff.next_backoff() {
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

    /// Determine if an error should be retried based on retry configuration
    ///
    /// Uses the retry configuration to decide whether a specific error warrants
    /// another attempt. This implements smart retry logic that:
    /// - NEVER retries rate limits (must be handled globally)
    /// - NEVER retries client errors (4xx - invalid input)
    /// - CONDITIONALLY retries server errors (5xx - based on config)
    /// - ALWAYS retries network/timeout errors (transient failures)
    ///
    /// # Arguments
    ///
    /// * `error` - The error to evaluate
    /// * `attempts` - Number of attempts made so far
    ///
    /// # Returns
    ///
    /// `true` if the error should be retried, `false` otherwise
    fn should_retry(&self, error: &OdosError, attempts: u32) -> bool {
        let retry_config = &self.config.retry_config;

        // Check attempt limit
        if attempts >= retry_config.max_retries {
            return false;
        }

        // Check custom predicate first
        if let Some(predicate) = retry_config.retry_predicate {
            return predicate(error);
        }

        // Default retry logic
        match error {
            // NEVER retry rate limits - application must handle globally
            OdosError::RateLimit { .. } => false,

            // NEVER retry client errors - invalid input
            OdosError::Api { status, .. } if status.is_client_error() => false,

            // MAYBE retry server errors - configurable
            OdosError::Api { status, .. } if status.is_server_error() => {
                retry_config.retry_server_errors
            }

            // ALWAYS retry network errors - transient
            OdosError::Http(err) => err.is_timeout() || err.is_connect() || err.is_request(),

            // ALWAYS retry timeout errors
            OdosError::Timeout(_) => true,

            // Don't retry anything else by default
            _ => false,
        }
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

/// Parse structured error response from Odos API
///
/// Attempts to parse the response body as a structured error JSON.
/// Returns the error message, optional error code, and optional trace ID.
/// Falls back to the raw body text if JSON parsing fails.
async fn parse_error_response(
    response: Response,
) -> (
    String,
    Option<OdosErrorCode>,
    Option<crate::error_code::TraceId>,
) {
    // Get the response body as text
    let body_text = match response.text().await {
        Ok(text) => text,
        Err(e) => return (format!("Failed to read response body: {}", e), None, None),
    };

    // Try to parse as structured error JSON
    match serde_json::from_str::<OdosApiErrorResponse>(&body_text) {
        Ok(error_response) => {
            // Successfully parsed structured error
            let error_code = OdosErrorCode::from(error_response.error_code);
            (
                error_response.detail,
                Some(error_code),
                Some(error_response.trace_id),
            )
        }
        Err(_) => {
            // Failed to parse as structured error, return raw body
            (body_text, None, None)
        }
    }
}

impl Default for OdosHttpClient {
    /// Creates a default HTTP client with standard configuration.
    ///
    /// # Panics
    ///
    /// Panics if the underlying HTTP client cannot be initialized.
    /// This should only fail in extremely rare cases such as:
    /// - TLS initialization failure
    /// - System resource exhaustion
    /// - Invalid system configuration
    ///
    /// In practice, this almost never fails and is safe for most use cases.
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

    /// Helper to create a test client with custom config
    fn create_test_client(max_retries: u32, timeout_ms: u64) -> OdosHttpClient {
        let config = ClientConfig {
            timeout: Duration::from_millis(timeout_ms),
            retry_config: RetryConfig {
                max_retries,
                initial_backoff_ms: 10,
                ..Default::default()
            },
            ..Default::default()
        };
        OdosHttpClient::with_config(config).unwrap()
    }

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.retry_config.max_retries, 3);
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
            retry_config: RetryConfig {
                max_retries: 5,
                ..Default::default()
            },
            ..Default::default()
        };
        let client = OdosHttpClient::with_config(config.clone());
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.config().timeout, Duration::from_secs(60));
        assert_eq!(client.config().retry_config.max_retries, 5);
    }

    #[tokio::test]
    async fn test_rate_limit_with_retry_after() {
        let mock_server = MockServer::start().await;

        // Mock returns 429 with Retry-After: 1 second
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(
                ResponseTemplate::new(429)
                    .set_body_string("Rate limit exceeded")
                    .insert_header("retry-after", "1"),
            )
            .expect(1) // Should only be called once (no retries)
            .mount(&mock_server)
            .await;

        let client = create_test_client(3, 30000);
        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        // Rate limits should return immediately without retry
        assert!(
            response.is_err(),
            "Rate limit should return error immediately"
        );

        if let Err(OdosError::RateLimit {
            message,
            retry_after,
        }) = response
        {
            assert!(message.contains("Rate limit"));
            assert_eq!(retry_after, Some(Duration::from_secs(1)));
        } else {
            panic!("Expected RateLimit error, got: {response:?}");
        }
    }

    #[tokio::test]
    async fn test_rate_limit_without_retry_after() {
        let mock_server = MockServer::start().await;

        // Mock returns 429 without Retry-After header
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(429).set_body_string("Rate limit exceeded"))
            .expect(1) // Should only be called once (no retries)
            .mount(&mock_server)
            .await;

        let client = create_test_client(3, 30000);
        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        // Rate limits should return immediately without retry
        assert!(
            response.is_err(),
            "Rate limit should return error immediately"
        );

        if let Err(OdosError::RateLimit {
            message,
            retry_after,
        }) = response
        {
            assert!(message.contains("Rate limit"));
            assert_eq!(retry_after, None);
        } else {
            panic!("Expected RateLimit error, got: {response:?}");
        }
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

        let client = OdosHttpClient::with_config(ClientConfig::default()).unwrap();

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

        let client = create_test_client(2, 30000);

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

        let client = create_test_client(2, 100);

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
        let client = create_test_client(2, 100);

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
            retry_config: RetryConfig {
                max_retries: 5,
                ..Default::default()
            },
            ..Default::default()
        };
        let client = OdosHttpClient::with_config(config.clone()).unwrap();

        // Test config() accessor
        assert_eq!(client.config().timeout, Duration::from_secs(45));
        assert_eq!(client.config().retry_config.max_retries, 5);

        // Test inner() accessor - just verify it returns a Client
        let _inner: &reqwest::Client = client.inner();
    }

    #[test]
    fn test_default_client() {
        let client = OdosHttpClient::default();

        // Should use default config
        assert_eq!(client.config().timeout, Duration::from_secs(30));
        assert_eq!(client.config().retry_config.max_retries, 3);
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

    #[tokio::test]
    async fn test_rate_limit_with_retry_after_zero() {
        let mock_server = MockServer::start().await;

        // Mock returns 429 with Retry-After: 0
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(
                ResponseTemplate::new(429)
                    .set_body_string("Rate limit exceeded")
                    .insert_header("retry-after", "0"),
            )
            .expect(1) // Should only be called once (no retries)
            .mount(&mock_server)
            .await;

        let client = create_test_client(3, 30000);
        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        // Rate limits should return immediately without retry (even with Retry-After: 0)
        assert!(
            response.is_err(),
            "Rate limit should return error immediately"
        );

        if let Err(OdosError::RateLimit {
            message,
            retry_after,
        }) = response
        {
            assert!(message.contains("Rate limit"));
            assert_eq!(retry_after, Some(Duration::from_secs(0)));
        } else {
            panic!("Expected RateLimit error, got: {response:?}");
        }
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
    async fn test_parse_structured_error_response() {
        use crate::error_code::OdosErrorCode;

        // Create a mock response with structured error
        let error_json = r#"{
            "detail": "Error getting quote, please try again",
            "traceId": "10becdc8-a021-4491-8201-a17b657204e0",
            "errorCode": 2999
        }"#;

        let http_response = http::Response::builder()
            .status(500)
            .body(error_json)
            .unwrap();
        let response = reqwest::Response::from(http_response);

        let (message, code, trace_id) = parse_error_response(response).await;

        assert_eq!(message, "Error getting quote, please try again");
        assert!(code.is_some());
        assert_eq!(code.unwrap(), OdosErrorCode::AlgoInternal);
        assert!(trace_id.is_some());
        assert_eq!(
            trace_id.unwrap().to_string(),
            "10becdc8-a021-4491-8201-a17b657204e0"
        );
    }

    #[tokio::test]
    async fn test_parse_unstructured_error_response() {
        // Create a mock response with plain text error
        let http_response = http::Response::builder()
            .status(500)
            .body("Internal server error")
            .unwrap();
        let response = reqwest::Response::from(http_response);

        let (message, code, trace_id) = parse_error_response(response).await;

        assert_eq!(message, "Internal server error");
        assert!(code.is_none());
        assert!(trace_id.is_none());
    }

    #[tokio::test]
    async fn test_api_error_with_structured_response() {
        let mock_server = MockServer::start().await;

        let error_json = r#"{
            "detail": "Invalid chain ID",
            "traceId": "a0b1c2d3-e4f5-6789-0abc-def123456789",
            "errorCode": 4001
        }"#;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(400).set_body_string(error_json))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = create_test_client(0, 30000);
        let response = client
            .execute_with_retry(|| client.inner().get(format!("{}/test", mock_server.uri())))
            .await;

        assert!(response.is_err());
        if let Err(e) = response {
            // Check that it's an API error
            assert!(matches!(e, OdosError::Api { .. }));

            // Check error code
            let error_code = e.error_code();
            assert!(error_code.is_some());
            assert!(error_code.unwrap().is_invalid_chain_id());

            // Check trace ID
            let trace_id = e.trace_id();
            assert!(trace_id.is_some());
        } else {
            panic!("Expected error, got success");
        }
    }

    #[tokio::test]
    async fn test_client_config_failure() {
        // Test that invalid configs are handled gracefully
        // Using an extremely high connection limit
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
