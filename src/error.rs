use alloy_primitives::hex;
use reqwest::StatusCode;
use thiserror::Error;

use crate::OdosChainError;

/// Result type alias for Odos SDK operations
pub type Result<T> = std::result::Result<T, OdosError>;

/// Comprehensive error types for the Odos SDK
///
/// This enum provides detailed error types for different failure scenarios,
/// allowing users to handle specific error conditions appropriately.
///
/// ## Error Categories
///
/// - **Network Errors**: HTTP, timeout, and connectivity issues
/// - **API Errors**: Responses from the Odos service indicating various failures
/// - **Input Errors**: Invalid parameters or missing required data
/// - **System Errors**: Circuit breaker, rate limiting, and internal failures
///
/// ## Retryable Errors
///
/// Some error types are marked as retryable (see [`OdosError::is_retryable`]):
/// - Timeout errors
/// - Certain HTTP errors (5xx status codes, connection issues)
/// - Rate limiting errors
/// - Some API errors (server errors, rate limits)
///
/// ## Examples
///
/// ```rust
/// use odos_sdk::{OdosError, Result};
/// use reqwest::StatusCode;
///
/// // Create different error types
/// let api_error = OdosError::api_error(StatusCode::BAD_REQUEST, "Invalid input".to_string());
/// let timeout_error = OdosError::timeout_error("Request timed out");
/// let circuit_breaker_error = OdosError::circuit_breaker_error("Circuit breaker is open");
///
/// // Check if errors are retryable
/// assert!(!api_error.is_retryable());  // 4xx errors are not retryable
/// assert!(timeout_error.is_retryable()); // Timeouts are retryable
/// assert!(!circuit_breaker_error.is_retryable()); // Circuit breaker prevents retries
///
/// // Get error categories for metrics
/// assert_eq!(api_error.category(), "api");
/// assert_eq!(timeout_error.category(), "timeout");
/// assert_eq!(circuit_breaker_error.category(), "circuit_breaker");
/// ```
#[derive(Error, Debug)]
pub enum OdosError {
    /// HTTP request errors
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// API errors returned by the Odos service
    #[error("Odos API error (status: {status}): {message}")]
    Api { status: StatusCode, message: String },

    /// JSON serialization/deserialization errors
    #[error("JSON processing error: {0}")]
    Json(#[from] serde_json::Error),

    /// Hex decoding errors
    #[error("Hex decoding error: {0}")]
    Hex(#[from] hex::FromHexError),

    /// Invalid input parameters
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Missing required data
    #[error("Missing required data: {0}")]
    MissingData(String),

    /// Chain not supported
    #[error("Chain not supported: {chain_id}")]
    UnsupportedChain { chain_id: u64 },

    /// Contract interaction errors
    #[error("Contract error: {0}")]
    Contract(String),

    /// Transaction assembly errors
    #[error("Transaction assembly failed: {0}")]
    TransactionAssembly(String),

    /// Quote request errors
    #[error("Quote request failed: {0}")]
    QuoteRequest(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Circuit breaker is open
    ///
    /// This error occurs when the circuit breaker has detected too many failures
    /// and has opened to prevent further requests. The circuit breaker will
    /// automatically transition to half-open state after a timeout period,
    /// allowing a limited number of requests to test if the service has recovered.
    ///
    /// ## When this occurs:
    /// - When the failure count exceeds the configured threshold
    /// - During the open state of the circuit breaker
    /// - Before the reset timeout has elapsed
    ///
    /// ## How to handle:
    /// - Wait for the circuit breaker to reset (typically 60 seconds by default)
    /// - Check the circuit breaker status using [`OdosSorV2::circuit_breaker_status`]
    /// - Implement exponential backoff in your retry logic
    /// - Consider using alternative service endpoints if available
    ///
    /// This error is **not retryable** as the circuit breaker is specifically
    /// designed to prevent additional load on a failing service.
    #[error("Circuit breaker is open: {0}")]
    CircuitBreakerOpen(String),

    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl OdosError {
    /// Create an API error from response
    pub fn api_error(status: StatusCode, message: String) -> Self {
        Self::Api { status, message }
    }

    /// Create an invalid input error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    /// Create a missing data error
    pub fn missing_data(message: impl Into<String>) -> Self {
        Self::MissingData(message.into())
    }

    /// Create an unsupported chain error
    pub fn unsupported_chain(chain_id: u64) -> Self {
        Self::UnsupportedChain { chain_id }
    }

    /// Create a contract error
    pub fn contract_error(message: impl Into<String>) -> Self {
        Self::Contract(message.into())
    }

    /// Create a transaction assembly error
    pub fn transaction_assembly_error(message: impl Into<String>) -> Self {
        Self::TransactionAssembly(message.into())
    }

    /// Create a quote request error
    pub fn quote_request_error(message: impl Into<String>) -> Self {
        Self::QuoteRequest(message.into())
    }

    /// Create a configuration error
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::Configuration(message.into())
    }

    /// Create a timeout error
    pub fn timeout_error(message: impl Into<String>) -> Self {
        Self::Timeout(message.into())
    }

    /// Create a rate limit error
    pub fn rate_limit_error(message: impl Into<String>) -> Self {
        Self::RateLimit(message.into())
    }

    /// Create a circuit breaker error
    pub fn circuit_breaker_error(message: impl Into<String>) -> Self {
        Self::CircuitBreakerOpen(message.into())
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            // HTTP errors that are typically retryable
            OdosError::Http(err) => {
                // Timeout, connection errors, etc.
                err.is_timeout() || err.is_connect() || err.is_request()
            }
            // API errors that might be retryable
            OdosError::Api { status, .. } => {
                matches!(
                    *status,
                    StatusCode::TOO_MANY_REQUESTS
                        | StatusCode::INTERNAL_SERVER_ERROR
                        | StatusCode::BAD_GATEWAY
                        | StatusCode::SERVICE_UNAVAILABLE
                        | StatusCode::GATEWAY_TIMEOUT
                )
            }
            // Other retryable errors
            OdosError::Timeout(_) => true,
            OdosError::RateLimit(_) => true,
            // Non-retryable errors
            OdosError::Json(_)
            | OdosError::Hex(_)
            | OdosError::InvalidInput(_)
            | OdosError::MissingData(_)
            | OdosError::UnsupportedChain { .. }
            | OdosError::Contract(_)
            | OdosError::TransactionAssembly(_)
            | OdosError::QuoteRequest(_)
            | OdosError::Configuration(_)
            | OdosError::CircuitBreakerOpen(_)
            | OdosError::Internal(_) => false,
        }
    }

    /// Get the error category for metrics
    pub fn category(&self) -> &'static str {
        match self {
            OdosError::Http(_) => "http",
            OdosError::Api { .. } => "api",
            OdosError::Json(_) => "json",
            OdosError::Hex(_) => "hex",
            OdosError::InvalidInput(_) => "invalid_input",
            OdosError::MissingData(_) => "missing_data",
            OdosError::UnsupportedChain { .. } => "unsupported_chain",
            OdosError::Contract(_) => "contract",
            OdosError::TransactionAssembly(_) => "transaction_assembly",
            OdosError::QuoteRequest(_) => "quote_request",
            OdosError::Configuration(_) => "configuration",
            OdosError::Timeout(_) => "timeout",
            OdosError::RateLimit(_) => "rate_limit",
            OdosError::CircuitBreakerOpen(_) => "circuit_breaker",
            OdosError::Internal(_) => "internal",
        }
    }
}

// Compatibility with anyhow for gradual migration
impl From<anyhow::Error> for OdosError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

// Convert chain errors to appropriate error types
impl From<OdosChainError> for OdosError {
    fn from(err: OdosChainError) -> Self {
        match err {
            OdosChainError::V2NotAvailable { chain } => {
                Self::contract_error(format!("V2 router not available on chain: {chain}"))
            }
            OdosChainError::V3NotAvailable { chain } => {
                Self::contract_error(format!("V3 router not available on chain: {chain}"))
            }
            OdosChainError::UnsupportedChain { chain } => {
                Self::contract_error(format!("Unsupported chain: {chain}"))
            }
            OdosChainError::InvalidAddress { address } => {
                Self::invalid_input(format!("Invalid address format: {address}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    #[test]
    fn test_retryable_errors() {
        // HTTP timeout should be retryable
        let timeout_err = OdosError::timeout_error("Request timed out");
        assert!(timeout_err.is_retryable());

        // API 500 error should be retryable
        let api_err = OdosError::api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Server error".to_string(),
        );
        assert!(api_err.is_retryable());

        // Invalid input should not be retryable
        let invalid_err = OdosError::invalid_input("Bad parameter");
        assert!(!invalid_err.is_retryable());

        // Rate limit should be retryable
        let rate_limit_err = OdosError::rate_limit_error("Too many requests");
        assert!(rate_limit_err.is_retryable());
    }

    #[test]
    fn test_error_categories() {
        let api_err = OdosError::api_error(StatusCode::BAD_REQUEST, "Bad request".to_string());
        assert_eq!(api_err.category(), "api");

        let timeout_err = OdosError::timeout_error("Timeout");
        assert_eq!(timeout_err.category(), "timeout");

        let invalid_err = OdosError::invalid_input("Invalid");
        assert_eq!(invalid_err.category(), "invalid_input");
    }
}
