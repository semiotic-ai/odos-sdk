use std::time::Duration;

use backoff::{backoff::Backoff, ExponentialBackoff};
use reqwest::{Client, RequestBuilder, Response};
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
        // Configure backoff strategy
        let mut backoff = ExponentialBackoff {
            initial_interval: self.config.initial_retry_delay,
            max_interval: self.config.max_retry_delay,
            max_elapsed_time: Some(self.config.timeout),
            ..Default::default()
        };

        let mut attempt = 0;
        let mut last_error = None;

        loop {
            if attempt >= self.config.max_retries {
                break;
            }

            attempt += 1;
            debug!(attempt = attempt, "Executing HTTP request");

            // Execute request with timeout
            let request = request_builder_fn().build().map_err(OdosError::Http)?;
            let request_timeout = timeout(self.config.timeout, self.client.execute(request));

            match request_timeout.await {
                Ok(Ok(response)) => {
                    // Check if response indicates success
                    if response.status().is_success() {
                        debug!(attempt = attempt, status = %response.status(), "Request successful");
                        return Ok(response);
                    } else {
                        // API error - check if retryable
                        let status = response.status();
                        let error_text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());
                        let error = OdosError::api_error(status, error_text);

                        if !error.is_retryable() {
                            return Err(error);
                        }

                        warn!(
                            attempt = attempt,
                            status = %status,
                            "Request failed with retryable error, retrying"
                        );
                        last_error = Some(error);
                    }
                }
                Ok(Err(reqwest_error)) => {
                    let error = OdosError::Http(reqwest_error);
                    if !error.is_retryable() {
                        return Err(error);
                    }

                    warn!(
                        attempt = attempt,
                        error = %error,
                        "Request failed with retryable error, retrying"
                    );
                    last_error = Some(error);
                }
                Err(_) => {
                    // Timeout
                    let error = OdosError::timeout_error("Request timed out");
                    warn!(
                        attempt = attempt,
                        timeout = ?self.config.timeout,
                        "Request timed out, retrying"
                    );
                    last_error = Some(error);
                }
            }

            // Wait before retry
            if attempt < self.config.max_retries {
                if let Some(delay) = backoff.next_backoff() {
                    debug!(delay = ?delay, "Waiting before retry");
                    tokio::time::sleep(delay).await;
                } else {
                    break; // Backoff expired
                }
            }
        }

        // All retries exhausted
        Err(last_error.unwrap_or_else(|| OdosError::internal_error("All retry attempts failed")))
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

impl Default for OdosHttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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
}
