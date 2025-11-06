use std::fmt::Display;

use alloy_primitives::{Address, Bytes, U256};
use bon::Builder;
use serde::{Deserialize, Serialize};
use tracing::debug;
use url::Url;

use crate::{
    error_code::TraceId,
    OdosError,
    OdosRouterV2::{inputTokenInfo, outputTokenInfo, swapTokenInfo},
    OdosV2Router::{swapCall, OdosV2RouterCalls},
    Result,
};

/// Base endpoint for the Odos API
///
/// Odos provides two API endpoints:
/// - **Public**: Standard API available to all users at <https://api.odos.xyz>
/// - **Enterprise**: Premium API with enhanced features at <https://enterprise-api.odos.xyz>
///
/// The endpoint base is separated from the API version ([`EndpointVersion`]) to allow
/// flexible configuration of both base URL and API version independently.
///
/// This enum derives [`Copy`] as it's a lightweight configuration value that's
/// frequently passed around and copied.
///
/// # Examples
///
/// ```rust
/// use odos_sdk::{EndpointBase, EndpointVersion};
///
/// let public_endpoint = EndpointBase::Public;
/// let v2_url = public_endpoint.quote_url(EndpointVersion::V2);
/// assert_eq!(v2_url.as_str(), "https://api.odos.xyz/sor/quote/v2");
///
/// let v3_url = public_endpoint.quote_url(EndpointVersion::V3);
/// assert_eq!(v3_url.as_str(), "https://api.odos.xyz/sor/quote/v3");
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum EndpointBase {
    /// Public API endpoint <https://docs.odos.xyz/build/api-docs>
    ///
    /// Standard API available to all users. Suitable for most use cases.
    Public,
    /// Enterprise API endpoint <https://docs.odos.xyz/build/enterprise-api>
    ///
    /// Premium API with enhanced features, higher rate limits, and dedicated support.
    /// Requires an API key obtained through the Odos Enterprise program.
    Enterprise,
}

impl EndpointBase {
    /// Get the base URL for the Odos API
    ///
    /// Returns the root URL for the selected endpoint without any path segments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::EndpointBase;
    ///
    /// let public = EndpointBase::Public;
    /// assert_eq!(public.base_url().as_str(), "https://api.odos.xyz/");
    ///
    /// let enterprise = EndpointBase::Enterprise;
    /// assert_eq!(enterprise.base_url().as_str(), "https://enterprise-api.odos.xyz/");
    /// ```
    pub fn base_url(&self) -> Url {
        match self {
            EndpointBase::Public => Url::parse("https://api.odos.xyz/").unwrap(),
            EndpointBase::Enterprise => Url::parse("https://enterprise-api.odos.xyz/").unwrap(),
        }
    }

    /// Get the quote URL for the Odos API with a specific version
    ///
    /// Constructs the full URL for the quote endpoint by combining the base URL
    /// with the appropriate version path.
    ///
    /// # Arguments
    ///
    /// * `version` - The API version to use (V2 or V3)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::{EndpointBase, EndpointVersion};
    ///
    /// let endpoint = EndpointBase::Public;
    ///
    /// let v2_url = endpoint.quote_url(EndpointVersion::V2);
    /// assert_eq!(v2_url.as_str(), "https://api.odos.xyz/sor/quote/v2");
    ///
    /// let v3_url = endpoint.quote_url(EndpointVersion::V3);
    /// assert_eq!(v3_url.as_str(), "https://api.odos.xyz/sor/quote/v3");
    /// ```
    pub fn quote_url(&self, version: EndpointVersion) -> Url {
        match version {
            EndpointVersion::V2 => self.base_url().join("sor/quote/v2").unwrap(),
            EndpointVersion::V3 => self.base_url().join("sor/quote/v3").unwrap(),
        }
    }

    /// Get the assemble URL for the Odos API
    ///
    /// The assemble endpoint is version-independent and constructs transaction data
    /// from a previously obtained quote path ID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::EndpointBase;
    ///
    /// let endpoint = EndpointBase::Public;
    /// let assemble_url = endpoint.assemble_url();
    /// assert_eq!(assemble_url.as_str(), "https://api.odos.xyz/sor/assemble");
    /// ```
    pub fn assemble_url(&self) -> Url {
        self.base_url().join("sor/assemble").unwrap()
    }
}

/// Version of the Odos API endpoint
///
/// Odos provides multiple API versions with different features and response formats:
/// - **V2**: Stable production version with comprehensive swap routing
/// - **V3**: Latest version with enhanced features and optimizations
///
/// The version is specified independently from the endpoint base ([`EndpointBase`])
/// to allow flexible configuration. Both public and enterprise endpoints support
/// all API versions.
///
/// This enum derives [`Copy`] as it's a lightweight configuration value that's
/// frequently passed around and copied.
///
/// # Migration from Previous Versions
///
/// Prior to version 0.20.0, the client configuration stored concrete URLs
/// (`quote_url`, `assemble_url`). The refactor to `EndpointBase` + `EndpointVersion`
/// provides more flexibility and clearer separation of concerns.
///
/// **Old approach (pre-0.20.0):**
/// ```rust,ignore
/// let config = ClientConfig {
///     quote_url: Url::parse("https://api.odos.xyz/sor/quote/v2").unwrap(),
///     assemble_url: Url::parse("https://api.odos.xyz/sor/assemble").unwrap(),
///     ..Default::default()
/// };
/// ```
///
/// **New approach (0.20.0+):**
/// ```rust
/// use odos_sdk::{ClientConfig, EndpointBase, EndpointVersion};
///
/// let config = ClientConfig {
///     endpoint: EndpointBase::Public,
///     endpoint_version: EndpointVersion::V2,
///     ..Default::default()
/// };
/// ```
///
/// # Examples
///
/// ```rust
/// use odos_sdk::{ClientConfig, EndpointBase, EndpointVersion};
///
/// // Use V2 API (stable, recommended for production)
/// let config_v2 = ClientConfig {
///     endpoint: EndpointBase::Public,
///     endpoint_version: EndpointVersion::V2,
///     ..Default::default()
/// };
///
/// // Use V3 API (latest features)
/// let config_v3 = ClientConfig {
///     endpoint: EndpointBase::Public,
///     endpoint_version: EndpointVersion::V3,
///     ..Default::default()
/// };
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum EndpointVersion {
    /// API version 2 - Stable production version
    ///
    /// Recommended for most production use cases. Provides comprehensive
    /// swap routing with extensive DEX coverage.
    V2,
    /// API version 3 - Latest version with enhanced features
    ///
    /// Includes optimizations and new features. Check the Odos documentation
    /// for specific enhancements over V2.
    V3,
}

/// Input token for the Odos quote API
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputToken {
    // Haven't looked much into it, but there's trouble if you try to make this a `Address`
    token_address: String,
    // Odos API error message: "Input Amount should be positive integer in string form with < 64 digits[0x6]"
    amount: String,
}

impl InputToken {
    pub fn new(token_address: Address, amount: U256) -> Self {
        Self {
            token_address: token_address.to_string(),
            amount: amount.to_string(),
        }
    }
}

impl From<(Address, U256)> for InputToken {
    fn from((token_address, amount): (Address, U256)) -> Self {
        Self::new(token_address, amount)
    }
}

impl Display for InputToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "InputToken {{ token_address: {}, amount: {} }}",
            self.token_address, self.amount
        )
    }
}

/// Output token for the Odos quote API
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputToken {
    // Haven't looked much into it, but there's trouble if you try to make this a `Address`
    token_address: String,
    proportion: u32,
}

impl OutputToken {
    pub fn new(token_address: Address, proportion: u32) -> Self {
        Self {
            token_address: token_address.to_string(),
            proportion,
        }
    }
}

impl From<(Address, u32)> for OutputToken {
    fn from((token_address, proportion): (Address, u32)) -> Self {
        Self::new(token_address, proportion)
    }
}

impl Display for OutputToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OutputToken {{ token_address: {}, proportion: {} }}",
            self.token_address, self.proportion
        )
    }
}

/// Request to the Odos quote API: <https://docs.odos.xyz/build/api-docs>
#[derive(Builder, Clone, Debug, Default, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    chain_id: u64,
    input_tokens: Vec<InputToken>,
    output_tokens: Vec<OutputToken>,
    slippage_limit_percent: f64,
    // Haven't looked much into it, but there's trouble if you try to make this a `Address`
    user_addr: String,
    compact: bool,
    simple: bool,
    referral_code: u32,
    disable_rfqs: bool,
    #[builder(default)]
    source_blacklist: Vec<String>,
}

/// Single quote response from the Odos quote API: <https://docs.odos.xyz/build/api-docs>
#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleQuoteResponse {
    block_number: u64,
    data_gas_estimate: u64,
    gas_estimate: f64,
    gas_estimate_value: f64,
    gwei_per_gas: f64,
    in_amounts: Vec<String>,
    in_tokens: Vec<Address>,
    in_values: Vec<f64>,
    net_out_value: f64,
    out_amounts: Vec<String>,
    out_tokens: Vec<Address>,
    out_values: Vec<f64>,
    partner_fee_percent: f64,
    path_id: String,
    path_viz: Option<String>,
    percent_diff: f64,
    price_impact: f64,
}

impl SingleQuoteResponse {
    /// Get the data gas estimate of the quote
    pub fn data_gas_estimate(&self) -> u64 {
        self.data_gas_estimate
    }

    /// Get the block number of the quote
    pub fn get_block_number(&self) -> u64 {
        self.block_number
    }

    /// Get the gas estimate of the quote
    pub fn gas_estimate(&self) -> f64 {
        self.gas_estimate
    }

    /// Get the in amounts of the quote
    pub fn in_amounts_iter(&self) -> impl Iterator<Item = &String> {
        self.in_amounts.iter()
    }

    /// Get the in amount of the quote
    pub fn in_amount_u256(&self) -> Result<U256> {
        let amount_str = self
            .in_amounts_iter()
            .next()
            .ok_or_else(|| OdosError::missing_data("Missing input amount"))?;
        let amount: u128 = amount_str
            .parse()
            .map_err(|_| OdosError::invalid_input("Invalid input amount format"))?;
        Ok(U256::from(amount))
    }

    /// Get the out amount of the quote
    pub fn out_amount(&self) -> Option<&String> {
        self.out_amounts.first()
    }

    /// Get the out amounts of the quote
    pub fn out_amounts_iter(&self) -> impl Iterator<Item = &String> {
        self.out_amounts.iter()
    }

    /// Get the in tokens of the quote
    pub fn in_tokens_iter(&self) -> impl Iterator<Item = &Address> {
        self.in_tokens.iter()
    }

    /// Get the in token of the quote
    pub fn first_in_token(&self) -> Option<&Address> {
        self.in_tokens.first()
    }

    pub fn out_tokens_iter(&self) -> impl Iterator<Item = &Address> {
        self.out_tokens.iter()
    }

    /// Get the out token of the quote
    pub fn first_out_token(&self) -> Option<&Address> {
        self.out_tokens.first()
    }

    /// Get the out values of the quote
    pub fn out_values_iter(&self) -> impl Iterator<Item = &f64> {
        self.out_values.iter()
    }

    /// Get the path id of the quote
    pub fn path_id(&self) -> &str {
        &self.path_id
    }

    /// Get the path id as a vector of bytes
    pub fn path_definition_as_vec_u8(&self) -> Vec<u8> {
        self.path_id().as_bytes().to_vec()
    }

    /// Get the swap input token and amount
    pub fn swap_input_token_and_amount(&self) -> Result<(Address, U256)> {
        let input_token = *self
            .in_tokens_iter()
            .next()
            .ok_or_else(|| OdosError::missing_data("Missing input token"))?;
        let input_amount_in_u256 = self.in_amount_u256()?;

        Ok((input_token, input_amount_in_u256))
    }

    /// Get the price impact of the quote
    pub fn price_impact(&self) -> f64 {
        self.price_impact
    }
}

/// Error response from the Odos API
///
/// When the Odos API returns an error, it includes:
/// - `detail`: Human-readable error message
/// - `traceId`: UUID for tracking the error in Odos logs
/// - `errorCode`: Numeric error code indicating the specific error type
///
/// Example error response:
/// ```json
/// {
///   "detail": "Error getting quote, please try again",
///   "traceId": "10becdc8-a021-4491-8201-a17b657204e0",
///   "errorCode": 2999
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OdosApiErrorResponse {
    /// Human-readable error message
    pub detail: String,
    /// Trace ID for debugging (UUID)
    pub trace_id: TraceId,
    /// Numeric error code
    pub error_code: u16,
}

/// Swap inputs for the Odos assemble API
#[derive(Clone, Debug)]
pub struct SwapInputs {
    executor: Address,
    path_definition: Bytes,
    input_token_info: inputTokenInfo,
    output_token_info: outputTokenInfo,
    value_out_min: U256,
}

impl TryFrom<OdosV2RouterCalls> for SwapInputs {
    type Error = OdosError;

    fn try_from(swap: OdosV2RouterCalls) -> std::result::Result<Self, Self::Error> {
        match swap {
            OdosV2RouterCalls::swap(call) => {
                debug!(
                    swap_type = "V2Router",
                    input.token = %call.tokenInfo.inputToken,
                    input.amount_wei = %call.tokenInfo.inputAmount,
                    output.token = %call.tokenInfo.outputToken,
                    output.min_wei = %call.tokenInfo.outputMin,
                    executor = %call.executor,
                    "Extracting swap inputs from V2 router call"
                );

                let swapCall {
                    executor,
                    pathDefinition,
                    referralCode,
                    tokenInfo,
                } = call;

                let _referral_code = referralCode;

                let swapTokenInfo {
                    inputToken,
                    inputAmount,
                    inputReceiver,
                    outputMin,
                    outputQuote,
                    outputReceiver,
                    outputToken,
                } = tokenInfo;

                let _output_quote = outputQuote;

                Ok(Self {
                    executor,
                    path_definition: pathDefinition,
                    input_token_info: inputTokenInfo {
                        tokenAddress: inputToken,
                        amountIn: inputAmount,
                        receiver: inputReceiver,
                    },
                    output_token_info: outputTokenInfo {
                        tokenAddress: outputToken,
                        relativeValue: U256::from(1),
                        receiver: outputReceiver,
                    },
                    value_out_min: outputMin,
                })
            }
            _ => Err(OdosError::invalid_input("Unexpected OdosV2RouterCalls")),
        }
    }
}

impl SwapInputs {
    /// Get the executor of the swap
    pub fn executor(&self) -> Address {
        self.executor
    }

    /// Get the path definition of the swap
    pub fn path_definition(&self) -> &Bytes {
        &self.path_definition
    }

    /// Get the token address of the swap
    pub fn token_address(&self) -> Address {
        self.input_token_info.tokenAddress
    }

    /// Get the amount in of the swap
    pub fn amount_in(&self) -> U256 {
        self.input_token_info.amountIn
    }

    /// Get the receiver of the swap
    pub fn receiver(&self) -> Address {
        self.input_token_info.receiver
    }

    /// Get the relative value of the swap
    pub fn relative_value(&self) -> U256 {
        self.output_token_info.relativeValue
    }

    /// Get the output token address of the swap
    pub fn output_token_address(&self) -> Address {
        self.output_token_info.tokenAddress
    }

    /// Get the value out min of the swap
    pub fn value_out_min(&self) -> U256 {
        self.value_out_min
    }
}
