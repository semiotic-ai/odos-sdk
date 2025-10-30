#[cfg(test)]
mod tests {
    use crate::*;
    use std::time::Duration;

    /// Integration tests for the enhanced error handling and HTTP client
    /// These tests verify the interaction between different components

    #[tokio::test]
    async fn test_odos_sor_with_custom_config() {
        let config = ClientConfig {
            timeout: Duration::from_secs(10),
            retry_config: RetryConfig {
                max_retries: 2,
                ..Default::default()
            },
            ..Default::default()
        };

        let sor_client = OdosSorV2::with_config(config).expect("Failed to create SOR client");

        // Verify the client was created successfully
        assert_eq!(sor_client.config().timeout, Duration::from_secs(10));
        assert_eq!(sor_client.config().retry_config.max_retries, 2);
    }

    #[tokio::test]
    async fn test_odos_sor_default() {
        let sor_client = OdosSorV2::new().expect("Failed to create default SOR client");

        // Verify default configuration
        assert_eq!(sor_client.config().timeout, Duration::from_secs(30));
        assert_eq!(sor_client.config().retry_config.max_retries, 3);
    }

    #[test]
    fn test_error_type_conversions() {
        // Test that our error types work correctly
        let api_error = OdosError::api_error(
            reqwest::StatusCode::BAD_REQUEST,
            "Invalid request".to_string(),
        );
        assert_eq!(api_error.category(), "api");
        assert!(!api_error.is_retryable());

        let timeout_error = OdosError::timeout_error("Request timed out");
        assert_eq!(timeout_error.category(), "timeout");
        assert!(timeout_error.is_retryable());

        let rate_limit_error = OdosError::rate_limit_error("Too many requests");
        assert_eq!(rate_limit_error.category(), "rate_limit");
        // Rate limits are NOT retryable - must be handled globally
        assert!(!rate_limit_error.is_retryable());
    }

    #[test]
    fn test_error_creation_and_classification() {
        // Test error creation helper functions
        let missing_data_err = OdosError::missing_data("Test missing data");
        assert_eq!(missing_data_err.category(), "missing_data");
        assert!(!missing_data_err.is_retryable());

        let invalid_input_err = OdosError::invalid_input("Test invalid input");
        assert_eq!(invalid_input_err.category(), "invalid_input");
        assert!(!invalid_input_err.is_retryable());

        let contract_err = OdosError::contract_error("Test contract error");
        assert_eq!(contract_err.category(), "contract");
        assert!(!contract_err.is_retryable());

        let assembly_err = OdosError::transaction_assembly_error("Test assembly error");
        assert_eq!(assembly_err.category(), "transaction_assembly");
        assert!(!assembly_err.is_retryable());
    }

    #[test]
    fn test_quote_request_builder() {
        use alloy_primitives::{Address, U256};

        let input_token = InputToken::new(Address::ZERO, U256::from(1000));
        let output_token = OutputToken::new(Address::ZERO, 1);

        let quote_request = QuoteRequest::builder()
            .chain_id(1)
            .input_tokens(vec![input_token])
            .output_tokens(vec![output_token])
            .slippage_limit_percent(1.0)
            .user_addr("0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0".to_string())
            .compact(false)
            .simple(false)
            .referral_code(0)
            .disable_rfqs(false)
            .build();

        // Just verify the builder worked - fields are private so we can't inspect them directly
        // This test validates that the builder pattern works correctly
        let _validated_request = quote_request;
    }

    #[test]
    fn test_error_retryability_logic() {
        // Test HTTP errors
        let timeout_err = OdosError::timeout_error("Timeout");
        assert!(timeout_err.is_retryable());

        let rate_limit_err = OdosError::rate_limit_error("Rate limited");
        // Rate limits are NOT retryable - must be handled globally
        assert!(!rate_limit_err.is_retryable());

        // Test API errors with different status codes
        let server_error = OdosError::api_error(
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            "Server error".to_string(),
        );
        assert!(server_error.is_retryable());

        let bad_request =
            OdosError::api_error(reqwest::StatusCode::BAD_REQUEST, "Bad request".to_string());
        assert!(!bad_request.is_retryable());

        // 429 errors should use RateLimit variant
        let too_many_requests = OdosError::rate_limit_error("Rate limited");
        // Rate limits are NOT retryable - must be handled globally
        assert!(!too_many_requests.is_retryable());

        // Test non-retryable errors
        let invalid_input = OdosError::invalid_input("Bad input");
        assert!(!invalid_input.is_retryable());

        let missing_data = OdosError::missing_data("Missing data");
        assert!(!missing_data.is_retryable());
    }
}
