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

    /// Get suggested retry delay for this error
    ///
    /// Returns a suggested delay before retrying the operation based on the error type:
    /// - **Rate Limit**: Returns the `retry_after` value from the API if available,
    ///   otherwise suggests 60 seconds. Note: Rate limits should be handled at the
    ///   application level with proper coordination.
    /// - **Timeout**: Suggests 1 second delay before retry
    /// - **HTTP Server Errors (5xx)**: Suggests 2 seconds with exponential backoff
    /// - **HTTP Connection Errors**: Suggests 500ms before retry
    /// - **Non-retryable Errors**: Returns `None`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::{OdosClient, QuoteRequest};
    /// use std::time::Duration;
    ///
    /// # async fn example(client: &OdosClient, request: &QuoteRequest) -> Result<(), Box<dyn std::error::Error>> {
    /// match client.quote(request).await {
    ///     Ok(quote) => { /* handle quote */ }
    ///     Err(e) => {
    ///         if let Some(delay) = e.suggested_retry_delay() {
    ///             println!("Retrying after {} seconds", delay.as_secs());
    ///             tokio::time::sleep(delay).await;
    ///             // Retry the operation...
    ///         } else {
    ///             println!("Error is not retryable: {}", e);
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn suggested_retry_delay(&self) -> Option<Duration> {
        match self {
            // Rate limit - use retry_after if available, otherwise 60s
            // Note: Rate limits should be handled globally, not per-request
            OdosError::RateLimit { retry_after, .. } => {
                Some(retry_after.unwrap_or(Duration::from_secs(60)))
            }
            // Timeout - short delay
            OdosError::Timeout(_) => Some(Duration::from_secs(1)),
            // API server errors - moderate delay
            OdosError::Api { status, .. } if status.is_server_error() => {
                Some(Duration::from_secs(2))
            }
            // HTTP errors - depends on error type
            OdosError::Http(err) => {
                if err.is_timeout() {
                    Some(Duration::from_secs(1))
                } else if err.is_connect() || err.is_request() {
                    Some(Duration::from_millis(500))
                } else {
                    None
                }
            }
            // All other errors are not retryable
            _ => None,
        }
    }

    /// Check if this is a client error (4xx status code)
    ///
    /// Returns `true` if this is an API error with a 4xx status code,
    /// indicating that the request was invalid and should not be retried
    /// without modification.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::OdosError;
    /// use reqwest::StatusCode;
    ///
    /// let error = OdosError::api_error(
    ///     StatusCode::BAD_REQUEST,
    ///     "Invalid chain ID".to_string()
    /// );
    ///
    /// assert!(error.is_client_error());
    /// assert!(!error.is_retryable());
    /// ```
    pub fn is_client_error(&self) -> bool {
        matches!(self, OdosError::Api { status, .. } if status.is_client_error())
    }

    /// Check if this is a server error (5xx status code)
    ///
    /// Returns `true` if this is an API error with a 5xx status code,
    /// indicating a server-side problem that may be resolved by retrying.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::OdosError;
    /// use reqwest::StatusCode;
    ///
    /// let error = OdosError::api_error(
    ///     StatusCode::INTERNAL_SERVER_ERROR,
    ///     "Server error".to_string()
    /// );
    ///
    /// assert!(error.is_server_error());
    /// assert!(error.is_retryable());
    /// ```
    pub fn is_server_error(&self) -> bool {
        matches!(self, OdosError::Api { status, .. } if status.is_server_error())
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

    #[test]
    fn test_suggested_retry_delay() {
        // Rate limit with retry-after
        let rate_limit_with_retry = OdosError::rate_limit_error_with_retry_after(
            "Rate limited",
            Some(Duration::from_secs(30)),
        );
        assert_eq!(
            rate_limit_with_retry.suggested_retry_delay(),
            Some(Duration::from_secs(30))
        );

        // Rate limit without retry-after (defaults to 60s)
        let rate_limit_no_retry = OdosError::rate_limit_error("Rate limited");
        assert_eq!(
            rate_limit_no_retry.suggested_retry_delay(),
            Some(Duration::from_secs(60))
        );

        // Timeout error
        let timeout_err = OdosError::timeout_error("Timeout");
        assert_eq!(
            timeout_err.suggested_retry_delay(),
            Some(Duration::from_secs(1))
        );

        // Server error
        let server_err = OdosError::api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Server error".to_string(),
        );
        assert_eq!(
            server_err.suggested_retry_delay(),
            Some(Duration::from_secs(2))
        );

        // Client error (not retryable)
        let client_err = OdosError::api_error(StatusCode::BAD_REQUEST, "Bad request".to_string());
        assert_eq!(client_err.suggested_retry_delay(), None);

        // Invalid input (not retryable)
        let invalid_err = OdosError::invalid_input("Invalid");
        assert_eq!(invalid_err.suggested_retry_delay(), None);
    }

    #[test]
    fn test_client_and_server_errors() {
        // Client error
        let client_err = OdosError::api_error(StatusCode::BAD_REQUEST, "Bad request".to_string());
        assert!(client_err.is_client_error());
        assert!(!client_err.is_server_error());

        // Server error
        let server_err = OdosError::api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Server error".to_string(),
        );
        assert!(!server_err.is_client_error());
        assert!(server_err.is_server_error());

        // Non-API error
        let other_err = OdosError::invalid_input("Invalid");
        assert!(!other_err.is_client_error());
        assert!(!other_err.is_server_error());
    }
}
