use alloy_primitives::{Address, Bytes, U256};
use bon::Builder;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    OdosRouterV2::{inputTokenInfo, outputTokenInfo, swapTokenInfo},
    OdosV2Router::{swapCall, OdosV2RouterCalls},
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputToken {
    // Haven't looked much into it, but there's trouble if you try to make this a `Address`
    token_address: String,
    // Odos API error message: "Input Amount should be positive integer in string form with < 64 digits[0x6]"
    amount: String,
}

impl InputToken {
    fn new(token_address: Address, amount: U256) -> Self {
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputToken {
    // Haven't looked much into it, but there's trouble if you try to make this a `Address`
    token_address: String,
    proportion: u32,
}

impl OutputToken {
    fn new(token_address: Address, proportion: u32) -> Self {
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

/// Request to the Odos quote API: <https://docs.odos.xyz/build/api-docs>
#[derive(Builder, Debug, Serialize)]
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
}

/// Single quote response from the Odos quote API: <https://docs.odos.xyz/build/api-docs>
#[derive(Debug, Clone, Deserialize, Serialize)]
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
    pub fn in_amount_u256(&self) -> anyhow::Result<U256> {
        let amount_str = self
            .in_amounts_iter()
            .next()
            .ok_or(anyhow::anyhow!("Missing input amount"))?;
        let amount: u128 = amount_str.parse()?;
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
    pub fn swap_input_token_and_amount(&self) -> anyhow::Result<(Address, U256)> {
        let input_token = *self
            .in_tokens_iter()
            .next()
            .ok_or(anyhow::anyhow!("Missing input token"))?;
        let input_amount_in_u256 = self.in_amount_u256()?;

        Ok((input_token, input_amount_in_u256))
    }

    /// Get the price impact of the quote
    pub fn price_impact(&self) -> f64 {
        self.price_impact
    }
}

#[derive(Debug, Clone)]
pub struct SwapInputs {
    executor: Address,
    path_definition: Bytes,
    input_token_info: inputTokenInfo,
    output_token_info: outputTokenInfo,
    value_out_min: U256,
}

impl TryFrom<OdosV2RouterCalls> for SwapInputs {
    type Error = anyhow::Error;

    fn try_from(swap: OdosV2RouterCalls) -> Result<Self, Self::Error> {
        match swap {
            OdosV2RouterCalls::swap(call) => {
                debug!(call = ?call);

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
            _ => Err(anyhow::anyhow!("Unexpected OdosV2RouterCalls")),
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
