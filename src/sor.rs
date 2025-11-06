use alloy_network::TransactionBuilder;
use alloy_primitives::{hex, Address};
use alloy_rpc_types::TransactionRequest;
use reqwest::Response;
use serde_json::Value;
use tracing::instrument;

use crate::{
    api::OdosApiErrorResponse, error_code::OdosErrorCode, parse_value, AssembleRequest,
    AssemblyResponse, ClientConfig, OdosError, OdosHttpClient, Result, RetryConfig, SwapContext,
};

use super::TransactionData;

use crate::{QuoteRequest, SingleQuoteResponse};

/// The Odos Smart Order Routing V2 API client
#[derive(Debug, Clone)]
pub struct OdosSorV2 {
    client: OdosHttpClient,
}

impl OdosSorV2 {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: OdosHttpClient::new()?,
        })
    }

    pub fn with_config(config: ClientConfig) -> Result<Self> {
        Ok(Self {
            client: OdosHttpClient::with_config(config)?,
        })
    }

    /// Create a client with custom retry configuration
    ///
    /// This is a convenience constructor that creates a client with the specified
    /// retry behavior while using default values for other configuration options.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::{OdosSorV2, RetryConfig};
    ///
    /// // No retries - handle all errors at application level
    /// let client = OdosSorV2::with_retry_config(RetryConfig::no_retries()).unwrap();
    ///
    /// // Conservative retries - only network errors
    /// let client = OdosSorV2::with_retry_config(RetryConfig::conservative()).unwrap();
    ///
    /// // Custom retry behavior
    /// let retry_config = RetryConfig {
    ///     max_retries: 5,
    ///     retry_server_errors: true,
    ///     ..Default::default()
    /// };
    /// let client = OdosSorV2::with_retry_config(retry_config).unwrap();
    /// ```
    pub fn with_retry_config(retry_config: RetryConfig) -> Result<Self> {
        let config = ClientConfig {
            retry_config,
            ..Default::default()
        };
        Self::with_config(config)
    }

    /// Get the client configuration
    pub fn config(&self) -> &ClientConfig {
        self.client.config()
    }

    /// Get a swap quote using Odos API
    ///
    /// Takes a [`QuoteRequest`] and returns a [`SingleQuoteResponse`].
    #[instrument(skip(self), level = "debug")]
    pub async fn get_swap_quote(
        &self,
        quote_request: &QuoteRequest,
    ) -> Result<SingleQuoteResponse> {
        let response = self
            .client
            .execute_with_retry(|| {
                let mut builder = self
                    .client
                    .inner()
                    .post(self.client.config().quote_url.clone())
                    .header("accept", "application/json")
                    .json(quote_request);

                // Add API key header if available
                if let Some(ref api_key) = self.client.config().api_key {
                    builder = builder.header("X-API-Key", api_key.as_str());
                }

                builder
            })
            .await?;

        if response.status().is_success() {
            let single_quote_response = response.json().await?;
            Ok(single_quote_response)
        } else {
            let status = response.status();

            // Try to parse structured error response
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read response body: {}", e));

            let (message, code, trace_id) =
                match serde_json::from_str::<OdosApiErrorResponse>(&body_text) {
                    Ok(error_response) => {
                        let error_code = OdosErrorCode::from(error_response.error_code);
                        (
                            error_response.detail,
                            error_code,
                            Some(error_response.trace_id),
                        )
                    }
                    Err(_) => (body_text, OdosErrorCode::Unknown(0), None),
                };

            Err(OdosError::api_error_with_code(
                status, message, code, trace_id,
            ))
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn get_assemble_response(
        &self,
        assemble_request: AssembleRequest,
    ) -> Result<Response> {
        self.client
            .execute_with_retry(|| {
                let mut builder = self
                    .client
                    .inner()
                    .post(self.client.config().assemble_url.clone())
                    .header("Content-Type", "application/json")
                    .json(&assemble_request);

                // Add API key header if available
                if let Some(ref api_key) = self.client.config().api_key {
                    builder = builder.header("X-API-Key", api_key.as_str());
                }

                builder
            })
            .await
    }

    /// Assemble transaction data from a quote using the Odos Assemble API.
    #[instrument(skip(self), level = "debug")]
    pub async fn assemble_tx_data(
        &self,
        signer_address: Address,
        output_recipient: Address,
        path_id: &str,
    ) -> Result<TransactionData> {
        let assemble_request = AssembleRequest {
            user_addr: signer_address.to_string(),
            path_id: path_id.to_string(),
            simulate: false,
            receiver: Some(output_recipient),
        };

        let response = self.get_assemble_response(assemble_request).await?;

        if !response.status().is_success() {
            let status = response.status();

            // Try to parse structured error response
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to get error message".to_string());

            let (message, code, trace_id) =
                match serde_json::from_str::<OdosApiErrorResponse>(&body_text) {
                    Ok(error_response) => {
                        let error_code = OdosErrorCode::from(error_response.error_code);
                        (
                            error_response.detail,
                            error_code,
                            Some(error_response.trace_id),
                        )
                    }
                    Err(_) => (body_text, OdosErrorCode::Unknown(0), None),
                };

            return Err(OdosError::api_error_with_code(
                status, message, code, trace_id,
            ));
        }

        let value: Value = response.json().await?;

        let AssemblyResponse { transaction, .. } = serde_json::from_value(value)?;

        Ok(transaction)
    }

    /// Build a base transaction from a swap using the Odos Assemble API,
    /// leaving gas parameters to be set by the caller.
    #[instrument(skip(self), level = "debug")]
    pub async fn build_base_transaction(&self, swap: &SwapContext) -> Result<TransactionRequest> {
        let TransactionData { data, value, .. } = self
            .assemble_tx_data(
                swap.signer_address(),
                swap.output_recipient(),
                swap.path_id(),
            )
            .await?;

        Ok(TransactionRequest::default()
            .with_input(hex::decode(&data)?)
            .with_value(parse_value(&value)?)
            .with_to(swap.router_address())
            .with_from(swap.signer_address()))
    }
}

impl Default for OdosSorV2 {
    /// Creates a default Odos SOR V2 client with standard configuration.
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
    /// See [`OdosHttpClient::default`] for more details.
    fn default() -> Self {
        Self::new().expect("Failed to create default OdosSorV2 client")
    }
}
