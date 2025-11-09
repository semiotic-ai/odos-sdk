use alloy_network::TransactionBuilder;
use alloy_primitives::{hex, Address};
use alloy_rpc_types::TransactionRequest;
use reqwest::Response;
use serde_json::Value;
use tracing::instrument;

use crate::{
    api::OdosApiErrorResponse, error_code::OdosErrorCode, parse_value, AssembleRequest,
    AssemblyRequest, AssemblyResponse, ClientConfig, OdosError, OdosHttpClient, Result,
    RetryConfig, SwapBuilder,
};

use super::TransactionData;

use crate::{QuoteRequest, SingleQuoteResponse};

/// The Odos API client
///
/// This is the primary interface for interacting with the Odos API. It provides
/// methods for obtaining swap quotes and assembling transactions.
///
/// # Architecture
///
/// The client is built on top of [`OdosHttpClient`], which handles:
/// - HTTP connection management and pooling
/// - Automatic retries with exponential backoff
/// - Rate limit handling
/// - Timeout management
///
/// # Examples
///
/// ## Basic usage with defaults
/// ```rust
/// use odos_sdk::OdosClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = OdosClient::new()?;
/// # Ok(())
/// # }
/// ```
///
/// ## Custom configuration
/// ```rust
/// use odos_sdk::{OdosClient, ClientConfig, Endpoint};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = ClientConfig {
///     endpoint: Endpoint::public_v3(),
///     ..Default::default()
/// };
/// let client = OdosClient::with_config(config)?;
/// # Ok(())
/// # }
/// ```
///
/// ## Using retry configuration
/// ```rust
/// use odos_sdk::{OdosClient, RetryConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Conservative retries - only network errors
/// let client = OdosClient::with_retry_config(RetryConfig::conservative())?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct OdosClient {
    client: OdosHttpClient,
}

impl OdosClient {
    /// Create a new Odos client with default configuration
    ///
    /// Uses default settings:
    /// - Public API endpoint
    /// - API version V2
    /// - 30 second timeout
    /// - 3 retry attempts with exponential backoff
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be initialized.
    /// This is rare and typically only occurs due to system resource issues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::OdosClient;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: OdosHttpClient::new()?,
        })
    }

    /// Create a new Odos SOR client with custom configuration
    ///
    /// Allows full control over client behavior including timeouts,
    /// retries, endpoint selection, and API version.
    ///
    /// # Arguments
    ///
    /// * `config` - The client configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be initialized.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::{OdosClient, ClientConfig, Endpoint};
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ClientConfig {
    ///     endpoint: Endpoint::enterprise_v3(),
    ///     ..Default::default()
    /// };
    /// let client = OdosClient::with_config(config)?;
    /// # Ok(())
    /// # }
    /// ```
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
    /// use odos_sdk::{OdosClient, RetryConfig};
    ///
    /// // No retries - handle all errors at application level
    /// let client = OdosClient::with_retry_config(RetryConfig::no_retries()).unwrap();
    ///
    /// // Conservative retries - only network errors
    /// let client = OdosClient::with_retry_config(RetryConfig::conservative()).unwrap();
    ///
    /// // Custom retry behavior
    /// let retry_config = RetryConfig {
    ///     max_retries: 5,
    ///     retry_server_errors: true,
    ///     ..Default::default()
    /// };
    /// let client = OdosClient::with_retry_config(retry_config).unwrap();
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

    /// Create a high-level swap builder
    ///
    /// This is the recommended way to build swaps for most use cases.
    /// It provides a simple, ergonomic API that handles the quote → assemble → build flow.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::{OdosClient, Chain, Slippage};
    /// use alloy_primitives::{address, U256};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    ///
    /// let tx = client
    ///     .swap()
    ///     .chain(Chain::ethereum())
    ///     .from_token(address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), U256::from(1_000_000))
    ///     .to_token(address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"))
    ///     .slippage(Slippage::percent(0.5)?)
    ///     .signer(address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0"))
    ///     .build_transaction()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn swap(&self) -> SwapBuilder<'_> {
        SwapBuilder::new(self)
    }

    /// Get a swap quote from the Odos API
    ///
    /// Requests a quote for swapping tokens on the configured chain.
    /// The quote includes routing information, price impact, gas estimates,
    /// and a path ID that can be used to assemble the transaction.
    ///
    /// # Arguments
    ///
    /// * `quote_request` - The quote request containing swap parameters
    ///
    /// # Returns
    ///
    /// Returns a [`SingleQuoteResponse`] containing:
    /// - Path ID for transaction assembly
    /// - Expected output amounts
    /// - Gas estimates
    /// - Price impact
    /// - Routing information
    ///
    /// # Errors
    ///
    /// This method can fail with various errors:
    /// - [`OdosError::Api`] - API returned an error (invalid input, unsupported chain, etc.)
    /// - [`OdosError::RateLimit`] - Rate limit exceeded
    /// - [`OdosError::Http`] - Network error
    /// - [`OdosError::Timeout`] - Request timeout
    ///
    /// Server errors (5xx) are automatically retried based on the retry configuration.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::{OdosClient, QuoteRequest, InputToken, OutputToken};
    /// use alloy_primitives::{address, U256};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    ///
    /// let quote_request = QuoteRequest::builder()
    ///     .chain_id(1) // Ethereum mainnet
    ///     .input_tokens(vec![InputToken::new(
    ///         address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), // USDC
    ///         U256::from(1000000) // 1 USDC (6 decimals)
    ///     )])
    ///     .output_tokens(vec![OutputToken::new(
    ///         address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"), // WETH
    ///         1 // 100% to WETH
    ///     )])
    ///     .slippage_limit_percent(0.5)
    ///     .user_addr(address!("0000000000000000000000000000000000000000"))
    ///     .compact(false)
    ///     .simple(false)
    ///     .referral_code(0)
    ///     .disable_rfqs(false)
    ///     .build();
    ///
    /// let quote = client.quote(&quote_request).await?;
    /// println!("Path ID: {}", quote.path_id());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), level = "debug")]
    pub async fn quote(&self, quote_request: &QuoteRequest) -> Result<SingleQuoteResponse> {
        let response = self
            .client
            .execute_with_retry(|| {
                let mut builder = self
                    .client
                    .inner()
                    .post(self.client.config().endpoint.quote_url())
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

    /// Deprecated: Use [`quote`](Self::quote) instead
    #[deprecated(since = "0.25.0", note = "Use `quote` instead")]
    pub async fn get_swap_quote(
        &self,
        quote_request: &QuoteRequest,
    ) -> Result<SingleQuoteResponse> {
        self.quote(quote_request).await
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
                    .post(self.client.config().endpoint.assemble_url())
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

    /// Assemble transaction data from a quote
    ///
    /// Takes a path ID from a quote response and assembles the complete
    /// transaction data needed to execute the swap on-chain.
    ///
    /// # Arguments
    ///
    /// * `signer_address` - Address that will sign and send the transaction
    /// * `output_recipient` - Address that will receive the output tokens
    /// * `path_id` - Path ID from a previous quote response
    ///
    /// # Returns
    ///
    /// Returns [`TransactionData`] containing:
    /// - Transaction calldata (`data`)
    /// - ETH value to send (`value`)
    /// - Target contract address (`to`)
    /// - Gas estimates
    ///
    /// # Errors
    ///
    /// - [`OdosError::Api`] - Invalid path ID, expired quote, or other API error
    /// - [`OdosError::RateLimit`] - Rate limit exceeded
    /// - [`OdosError::Http`] - Network error
    /// - [`OdosError::Timeout`] - Request timeout
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::OdosClient;
    /// use alloy_primitives::address;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// let path_id = "path_id_from_quote_response";
    ///
    /// let tx_data = client.assemble_tx_data(
    ///     address!("0000000000000000000000000000000000000001"),
    ///     address!("0000000000000000000000000000000000000001"),
    ///     path_id
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), level = "debug")]
    pub async fn assemble_tx_data(
        &self,
        signer_address: Address,
        output_recipient: Address,
        path_id: &str,
    ) -> Result<TransactionData> {
        let assemble_request = AssembleRequest {
            user_addr: signer_address,
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

    /// Assemble a transaction from an assembly request
    ///
    /// Assembles transaction data and constructs a [`TransactionRequest`] ready
    /// for gas parameter configuration and signing. This is a convenience method
    /// that combines [`assemble_tx_data`](Self::assemble_tx_data) with transaction
    /// request construction.
    ///
    /// # Arguments
    ///
    /// * `request` - The assembly request containing addresses and path ID
    ///
    /// # Returns
    ///
    /// Returns a [`TransactionRequest`] with:
    /// - `to`: Router contract address
    /// - `from`: Signer address
    /// - `data`: Encoded swap calldata
    /// - `value`: ETH amount to send
    ///
    /// Gas parameters (gas limit, gas price) are NOT set and must be configured
    /// by the caller before signing.
    ///
    /// # Errors
    ///
    /// - [`OdosError::Api`] - Invalid path ID or API error
    /// - [`OdosError::RateLimit`] - Rate limit exceeded
    /// - [`OdosError::Http`] - Network error
    /// - [`OdosError::Timeout`] - Request timeout
    /// - [`OdosError::Hex`] - Failed to decode transaction data
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::{OdosClient, AssemblyRequest};
    /// use alloy_primitives::{address, U256};
    /// use alloy_chains::NamedChain;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    ///
    /// let request = AssemblyRequest::builder()
    ///     .chain(NamedChain::Mainnet)
    ///     .signer_address(address!("0000000000000000000000000000000000000001"))
    ///     .output_recipient(address!("0000000000000000000000000000000000000001"))
    ///     .router_address(address!("0000000000000000000000000000000000000002"))
    ///     .token_address(address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"))
    ///     .token_amount(U256::from(1000000))
    ///     .path_id("path_id_from_quote".to_string())
    ///     .build();
    ///
    /// let mut tx_request = client.assemble(&request).await?;
    ///
    /// // Configure gas parameters before signing
    /// // tx_request = tx_request.with_gas_limit(300000);
    /// // tx_request = tx_request.with_max_fee_per_gas(...);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), level = "debug")]
    pub async fn assemble(&self, request: &AssemblyRequest) -> Result<TransactionRequest> {
        let TransactionData { data, value, .. } = self
            .assemble_tx_data(
                request.signer_address(),
                request.output_recipient(),
                request.path_id(),
            )
            .await?;

        Ok(TransactionRequest::default()
            .with_input(hex::decode(&data)?)
            .with_value(parse_value(&value)?)
            .with_to(request.router_address())
            .with_from(request.signer_address()))
    }

    /// Deprecated: Use [`assemble`](Self::assemble) instead
    #[deprecated(since = "0.25.0", note = "Use `assemble` instead")]
    pub async fn build_base_transaction(
        &self,
        swap: &AssemblyRequest,
    ) -> Result<TransactionRequest> {
        self.assemble(swap).await
    }
}

impl Default for OdosClient {
    /// Creates a default Odos client with standard configuration.
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
        Self::new().expect("Failed to create default OdosClient")
    }
}

/// Deprecated alias for [`OdosClient`]
///
/// This type alias is provided for backward compatibility.
/// Use [`OdosClient`] instead in new code.
#[deprecated(since = "0.25.0", note = "Use `OdosClient` instead")]
pub type OdosSor = OdosClient;
