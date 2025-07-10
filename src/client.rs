use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use backoff::{backoff::Backoff, ExponentialBackoff};
use reqwest::{Client, RequestBuilder, Response};
use tokio::time::timeout;
use tracing::{debug, info, instrument, warn};

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
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker reset timeout
    pub circuit_breaker_reset_timeout: Duration,
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
            circuit_breaker_threshold: 5,
            circuit_breaker_reset_timeout: Duration::from_secs(60),
            max_connections: 20,
            pool_idle_timeout: Duration::from_secs(90),
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Enhanced HTTP client with retry logic, timeouts, and circuit breaker
#[derive(Debug, Clone)]
pub struct OdosHttpClient {
    client: Client,
    config: ClientConfig,
    circuit_breaker: Arc<CircuitBreaker>,
}

#[derive(Debug)]
struct CircuitBreaker {
    state: std::sync::RwLock<CircuitBreakerState>,
    failure_count: AtomicU64,
    last_failure_time: std::sync::RwLock<Option<std::time::Instant>>,
    config: ClientConfig,
}

impl CircuitBreaker {
    fn new(config: ClientConfig) -> Self {
        Self {
            state: std::sync::RwLock::new(CircuitBreakerState::Closed),
            failure_count: AtomicU64::new(0),
            last_failure_time: std::sync::RwLock::new(None),
            config,
        }
    }

    fn can_execute(&self) -> Result<()> {
        let state = *self.state.read().unwrap();
        match state {
            CircuitBreakerState::Closed => Ok(()),
            CircuitBreakerState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = *self.last_failure_time.read().unwrap() {
                    if last_failure.elapsed() > self.config.circuit_breaker_reset_timeout {
                        *self.state.write().unwrap() = CircuitBreakerState::HalfOpen;
                        info!("Circuit breaker transitioning to half-open state");
                        Ok(())
                    } else {
                        Err(OdosError::circuit_breaker_error("Circuit breaker is open"))
                    }
                } else {
                    Ok(())
                }
            }
            CircuitBreakerState::HalfOpen => Ok(()),
        }
    }

    fn record_success(&self) {
        let current_state = *self.state.read().unwrap();
        match current_state {
            CircuitBreakerState::HalfOpen => {
                *self.state.write().unwrap() = CircuitBreakerState::Closed;
                self.failure_count.store(0, Ordering::SeqCst);
                info!("Circuit breaker closed after successful request");
            }
            CircuitBreakerState::Closed => {
                // Reset failure count on successful request
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitBreakerState::Open => {
                // Should not happen, but handle gracefully
                warn!("Recorded success while circuit breaker is open");
            }
        }
    }

    fn record_failure(&self) {
        let failure_count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure_time.write().unwrap() = Some(std::time::Instant::now());

        if failure_count >= self.config.circuit_breaker_threshold as u64 {
            *self.state.write().unwrap() = CircuitBreakerState::Open;
            warn!("Circuit breaker opened after {} failures", failure_count);
        }
    }
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

        Ok(Self {
            client,
            config: config.clone(),
            circuit_breaker: Arc::new(CircuitBreaker::new(config)),
        })
    }

    /// Execute a request with retry logic and circuit breaker
    #[instrument(skip(self, request_builder_fn), level = "debug")]
    pub async fn execute_with_retry<F>(&self, request_builder_fn: F) -> Result<Response>
    where
        F: Fn() -> RequestBuilder + Clone,
    {
        // Check circuit breaker
        self.circuit_breaker.can_execute()?;

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
                        self.circuit_breaker.record_success();
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
                            self.circuit_breaker.record_failure();
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
                        self.circuit_breaker.record_failure();
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
        self.circuit_breaker.record_failure();
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

    /// Get circuit breaker status
    pub fn circuit_breaker_status(&self) -> String {
        let state = *self.circuit_breaker.state.read().unwrap();
        let failure_count = self.circuit_breaker.failure_count.load(Ordering::SeqCst);
        format!("State: {state:?}, Failures: {failure_count}")
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
        assert_eq!(config.circuit_breaker_threshold, 5);
    }

    #[test]
    fn test_circuit_breaker_creation() {
        let config = ClientConfig::default();
        let cb = CircuitBreaker::new(config);
        assert_eq!(cb.failure_count.load(Ordering::SeqCst), 0);
        assert_eq!(*cb.state.read().unwrap(), CircuitBreakerState::Closed);
    }

    #[test]
    fn test_circuit_breaker_can_execute() {
        let config = ClientConfig::default();
        let cb = CircuitBreaker::new(config);
        assert!(cb.can_execute().is_ok());
    }

    #[test]
    fn test_circuit_breaker_record_success() {
        let config = ClientConfig::default();
        let cb = CircuitBreaker::new(config);
        cb.record_success();
        assert_eq!(cb.failure_count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_circuit_breaker_record_failure() {
        let config = ClientConfig {
            circuit_breaker_threshold: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        // First failure
        cb.record_failure();
        assert_eq!(cb.failure_count.load(Ordering::SeqCst), 1);
        assert_eq!(*cb.state.read().unwrap(), CircuitBreakerState::Closed);

        // Second failure should open circuit
        cb.record_failure();
        assert_eq!(cb.failure_count.load(Ordering::SeqCst), 2);
        assert_eq!(*cb.state.read().unwrap(), CircuitBreakerState::Open);
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
