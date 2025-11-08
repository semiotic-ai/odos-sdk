use std::time::Duration;

use alloy_primitives::hex;
use reqwest::StatusCode;
use thiserror::Error;

use crate::{
    error_code::{OdosErrorCode, TraceId},
    OdosChainError,
};

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
/// - **System Errors**: Rate limiting and internal failures
///
/// ## Retryable Errors
///
/// Some error types are marked as retryable (see [`OdosError::is_retryable`]):
/// - Timeout errors
/// - Certain HTTP errors (5xx status codes, connection issues)
/// - Some API errors (server errors)
///
/// **Note**: Rate limiting errors (429) are NOT retryable. Applications must handle
/// rate limits globally with proper coordination rather than retrying individual requests.
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
/// let rate_limit_error = OdosError::rate_limit_error("Too many requests");
///
/// // Check if errors are retryable
/// assert!(!api_error.is_retryable());  // 4xx errors are not retryable
/// assert!(timeout_error.is_retryable()); // Timeouts are retryable
/// assert!(!rate_limit_error.is_retryable()); // Rate limits are NOT retryable
///
/// // Get error categories for metrics
/// assert_eq!(api_error.category(), "api");
/// assert_eq!(timeout_error.category(), "timeout");
/// assert_eq!(rate_limit_error.category(), "rate_limit");
/// ```
#[derive(Error, Debug)]
pub enum OdosError {
    /// HTTP request errors
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// API errors returned by the Odos service
    #[error("Odos API error (status: {status}): {message}{}", trace_id.map(|tid| format!(" [trace: {}]", tid)).unwrap_or_default())]
    Api {
        status: StatusCode,
        message: String,
        code: OdosErrorCode,
        trace_id: Option<TraceId>,
    },

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
    ///
    /// Contains an optional `retry_after` duration from the Retry-After HTTP header,
    /// the error code from the Odos API, and an optional `trace_id` for debugging.
    #[error("Rate limit exceeded: {message}{}", trace_id.map(|tid| format!(" [trace: {}]", tid)).unwrap_or_default())]
    RateLimit {
        message: String,
        retry_after: Option<Duration>,
        code: OdosErrorCode,
        trace_id: Option<TraceId>,
    },

    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl OdosError {
    /// Create an API error from response (without error code or trace ID)
    pub fn api_error(status: StatusCode, message: String) -> Self {
        Self::Api {
            status,
            message,
            code: OdosErrorCode::Unknown(0),
            trace_id: None,
        }
    }

    /// Create an API error with error code and trace ID
    pub fn api_error_with_code(
        status: StatusCode,
        message: String,
        code: OdosErrorCode,
        trace_id: Option<TraceId>,
    ) -> Self {
        Self::Api {
            status,
            message,
            code,
            trace_id,
        }
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

    /// Create a rate limit error with optional retry-after duration
    pub fn rate_limit_error(message: impl Into<String>) -> Self {
        Self::RateLimit {
            message: message.into(),
            retry_after: None,
            code: OdosErrorCode::Unknown(429),
            trace_id: None,
        }
    }

    /// Create a rate limit error with retry-after duration
    pub fn rate_limit_error_with_retry_after(
        message: impl Into<String>,
        retry_after: Option<Duration>,
    ) -> Self {
        Self::RateLimit {
            message: message.into(),
            retry_after,
            code: OdosErrorCode::Unknown(429),
            trace_id: None,
        }
    }

    /// Create a rate limit error with all fields
    pub fn rate_limit_error_with_retry_after_and_trace(
        message: impl Into<String>,
        retry_after: Option<Duration>,
        code: OdosErrorCode,
        trace_id: Option<TraceId>,
    ) -> Self {
        Self::RateLimit {
            message: message.into(),
            retry_after,
            code,
            trace_id,
        }
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Check if the error is retryable
    ///
    /// For API errors, the retryability is determined by the error code.
    /// For Unknown error codes, falls back to HTTP status code checking.
    pub fn is_retryable(&self) -> bool {
        match self {
            // HTTP errors that are typically retryable
            OdosError::Http(err) => {
                // Timeout, connection errors, etc.
                err.is_timeout() || err.is_connect() || err.is_request()
            }
            // API errors - use error code retryability logic
            OdosError::Api { status, code, .. } => {
                // If we have a known error code, use its retryability logic
                if matches!(code, OdosErrorCode::Unknown(_)) {
                    // Fall back to status code checking for unknown error codes
                    matches!(
                        *status,
                        StatusCode::INTERNAL_SERVER_ERROR
                            | StatusCode::BAD_GATEWAY
                            | StatusCode::SERVICE_UNAVAILABLE
                            | StatusCode::GATEWAY_TIMEOUT
                    )
                } else {
                    code.is_retryable()
                }
            }
            // Other retryable errors
            OdosError::Timeout(_) => true,
            // NEVER retry rate limits - application must handle globally
            OdosError::RateLimit { .. } => false,
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
            | OdosError::Internal(_) => false,
        }
    }

    /// Check if this error is specifically a rate limit error
    ///
    /// This is a convenience method to help with error handling patterns.
    /// Rate limit errors indicate that the Odos API has rejected the request
    /// due to too many requests being made in a given time period.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::{OdosError, OdosSor, QuoteRequest};
    ///
    /// # async fn example(client: &OdosSor, request: &QuoteRequest) {
    /// match client.get_swap_quote(request).await {
    ///     Ok(quote) => { /* handle quote */ }
    ///     Err(e) if e.is_rate_limit() => {
    ///         // Specific handling for rate limits
    ///         eprintln!("Rate limited - consider backing off");
    ///     }
    ///     Err(e) => { /* handle other errors */ }
    /// }
    /// # }
    /// ```
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, OdosError::RateLimit { .. })
    }

    /// Get the retry-after duration for rate limit errors
    ///
    /// Returns `Some(duration)` if this is a rate limit error with a retry-after value,
    /// `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::OdosError;
    /// use std::time::Duration;
    ///
    /// let error = OdosError::rate_limit_error_with_retry_after(
    ///     "Rate limited",
    ///     Some(Duration::from_secs(30))
    /// );
    ///
    /// if let Some(duration) = error.retry_after() {
    ///     println!("Retry after {} seconds", duration.as_secs());
    /// }
    /// ```
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            OdosError::RateLimit { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    /// Get the Odos API error code if available
    ///
    /// Returns the strongly-typed error code for API and rate limit errors,
    /// or `None` for other error types.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::{OdosError, error_code::OdosErrorCode};
    /// use reqwest::StatusCode;
    ///
    /// let error = OdosError::api_error_with_code(
    ///     StatusCode::BAD_REQUEST,
    ///     "Invalid chain ID".to_string(),
    ///     OdosErrorCode::from(4001),
    ///     None
    /// );
    ///
    /// if let Some(code) = error.error_code() {
    ///     if code.is_invalid_chain_id() {
    ///         println!("Chain ID validation failed");
    ///     }
    /// }
    /// ```
    pub fn error_code(&self) -> Option<&OdosErrorCode> {
        match self {
            OdosError::Api { code, .. } => Some(code),
            OdosError::RateLimit { code, .. } => Some(code),
            _ => None,
        }
    }

    /// Get the Odos API trace ID if available
    ///
    /// Returns the trace ID for debugging API errors, or `None` for other error types
    /// or if the trace ID was not included in the API response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::OdosError;
    ///
    /// # fn handle_error(error: &OdosError) {
    /// if let Some(trace_id) = error.trace_id() {
    ///     eprintln!("Error trace ID for support: {}", trace_id);
    /// }
    /// # }
    /// ```
    pub fn trace_id(&self) -> Option<TraceId> {
        match self {
            OdosError::Api { trace_id, .. } => *trace_id,
            OdosError::RateLimit { trace_id, .. } => *trace_id,
            _ => None,
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
            OdosError::RateLimit { .. } => "rate_limit",
            OdosError::Internal(_) => "internal",
        }
    }
}

// Convert chain errors to appropriate error types
impl From<OdosChainError> for OdosError {
    fn from(err: OdosChainError) -> Self {
        match err {
            OdosChainError::LimitOrderNotAvailable { chain } => Self::contract_error(format!(
                "Limit Order router not available on chain: {chain}"
            )),
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

        // Rate limit should NOT be retryable (application must handle globally)
        let rate_limit_err = OdosError::rate_limit_error("Too many requests");
        assert!(!rate_limit_err.is_retryable());
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
