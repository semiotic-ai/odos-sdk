// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

//! Tool/runtime-friendly request and response types.
//!
//! This module provides a stable JSON boundary for tool runtimes, generated
//! integrations, MCP servers, CLIs, and backend services that need to request
//! quotes and build swap transactions without manually normalizing chain names,
//! addresses, slippage, and U256 amounts.

use alloy_primitives::{Address, U256};
use serde::{Deserialize, Serialize};

use crate::{
    parse_value, Chain, OdosClient, QuoteRequest, ReferralCode, Result, SingleQuoteResponse,
    Slippage, SwapBuilder, TransactionData,
};

/// Chain selector that accepts either a numeric chain ID or a common chain name.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ChainInput {
    /// Numeric EVM chain ID.
    Id(u64),
    /// Common chain name or alias such as `ethereum`, `mainnet`, `arb`, or `base`.
    Name(String),
}

impl ChainInput {
    /// Resolve this chain selector into a supported Odos chain.
    pub fn resolve(&self) -> Result<Chain> {
        match self {
            Self::Id(id) => Chain::from_chain_id(*id).map_err(|err| {
                crate::OdosError::invalid_input(format!("Unsupported Odos chain '{}': {}", id, err))
            }),
            Self::Name(name) => Chain::from_name(name).map_err(|err| {
                crate::OdosError::invalid_input(format!(
                    "Unsupported Odos chain '{}': {}",
                    name, err
                ))
            }),
        }
    }
}

/// Single-token swap request shape optimized for tool/runtime JSON boundaries.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapRequest {
    pub chain: ChainInput,
    pub from_token: String,
    pub from_amount: String,
    pub to_token: String,
    pub signer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slippage_percent: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referral_code: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub simple: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_rfqs: Option<bool>,
}

impl SwapRequest {
    /// Validate and normalize the request into typed Odos/alloy values.
    pub fn validate(&self) -> Result<ValidatedSwapRequest> {
        let chain = self.chain.resolve()?;
        let input_token = parse_address("fromToken", &self.from_token)?;
        let input_amount = parse_amount("fromAmount", &self.from_amount)?;
        let output_token = parse_address("toToken", &self.to_token)?;
        let signer = parse_address("signer", &self.signer)?;
        let recipient = self
            .recipient
            .as_deref()
            .map(|value| parse_address("recipient", value))
            .transpose()?
            .unwrap_or(signer);
        let slippage = resolve_slippage(self.slippage_percent, self.slippage_bps)?;
        let referral = self
            .referral_code
            .map(ReferralCode::new)
            .unwrap_or(ReferralCode::NONE);

        if input_amount.is_zero() {
            return Err(crate::OdosError::invalid_input(
                "fromAmount must be greater than zero",
            ));
        }

        if input_token == output_token {
            return Err(crate::OdosError::invalid_input(
                "fromToken and toToken must be different",
            ));
        }

        Ok(ValidatedSwapRequest {
            chain,
            input_token,
            input_amount,
            output_token,
            signer,
            recipient,
            slippage,
            referral,
            compact: self.compact.unwrap_or(false),
            simple: self.simple.unwrap_or(false),
            disable_rfqs: self.disable_rfqs.unwrap_or(false),
        })
    }
}

/// Validated single-token swap request with typed values ready for execution.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedSwapRequest {
    pub chain: Chain,
    pub input_token: Address,
    pub input_amount: U256,
    pub output_token: Address,
    pub signer: Address,
    pub recipient: Address,
    pub slippage: Slippage,
    pub referral: ReferralCode,
    pub compact: bool,
    pub simple: bool,
    pub disable_rfqs: bool,
}

impl ValidatedSwapRequest {
    /// Build an Odos quote request from the validated swap inputs.
    pub fn quote_request(&self) -> QuoteRequest {
        QuoteRequest::builder()
            .chain_id(self.chain.id())
            .input_tokens(vec![(self.input_token, self.input_amount).into()])
            .output_tokens(vec![(self.output_token, 1).into()])
            .slippage_limit_percent(self.slippage.as_percent())
            .user_addr(self.signer)
            .compact(self.compact)
            .simple(self.simple)
            .referral_code(self.referral.code())
            .disable_rfqs(self.disable_rfqs)
            .build()
    }

    /// Build a configured high-level swap builder from the validated request.
    pub fn swap_builder<'a>(&self, client: &'a OdosClient) -> SwapBuilder<'a> {
        client
            .swap()
            .chain(self.chain)
            .from_token(self.input_token, self.input_amount)
            .to_token(self.output_token)
            .slippage(self.slippage)
            .signer(self.signer)
            .recipient(self.recipient)
            .referral(self.referral)
            .compact(self.compact)
            .simple(self.simple)
            .disable_rfqs(self.disable_rfqs)
    }
}

/// Compact quote summary intended for tool outputs and confirmation prompts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteSummary {
    pub chain_id: u64,
    pub chain_name: String,
    pub signer: String,
    pub recipient: String,
    pub from_token: String,
    pub from_amount: String,
    pub to_token: String,
    pub to_amount: String,
    pub slippage_percent: f64,
    pub path_id: String,
    pub price_impact_percent: f64,
    pub gas_estimate: f64,
    pub gas_estimate_value: f64,
    pub net_out_value: f64,
    pub partner_fee_percent: f64,
    pub gwei_per_gas: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl QuoteSummary {
    fn from_quote(request: &ValidatedSwapRequest, quote: &SingleQuoteResponse) -> Self {
        let mut warnings = Vec::new();

        if quote.price_impact() >= 3.0 {
            warnings.push(format!(
                "High price impact detected ({:.2}%)",
                quote.price_impact()
            ));
        }

        if quote.gas_estimate_value() > quote.net_out_value() && quote.net_out_value() > 0.0 {
            warnings.push("Estimated gas cost exceeds quoted net output value".to_string());
        }

        if quote.out_amount().is_none() {
            warnings.push("Primary output amount was missing from the quote response".to_string());
        }

        Self {
            chain_id: request.chain.id(),
            chain_name: request.chain.to_string(),
            signer: request.signer.to_string(),
            recipient: request.recipient.to_string(),
            from_token: request.input_token.to_string(),
            from_amount: request.input_amount.to_string(),
            to_token: request.output_token.to_string(),
            to_amount: quote
                .out_amount()
                .cloned()
                .unwrap_or_else(|| "0".to_string()),
            slippage_percent: request.slippage.as_percent(),
            path_id: quote.path_id().to_string(),
            price_impact_percent: quote.price_impact(),
            gas_estimate: quote.gas_estimate(),
            gas_estimate_value: quote.gas_estimate_value(),
            net_out_value: quote.net_out_value(),
            partner_fee_percent: quote.partner_fee_percent(),
            gwei_per_gas: quote.gwei_per_gas(),
            warnings,
        }
    }
}

/// Transaction summary intended for tool/runtime outputs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSummary {
    pub to: String,
    pub from: String,
    pub data: String,
    pub value: String,
    pub gas: i128,
    pub gas_price: u128,
    pub chain_id: u64,
    pub nonce: u64,
}

impl From<TransactionData> for TransactionSummary {
    fn from(value: TransactionData) -> Self {
        Self {
            to: value.to.to_string(),
            from: value.from.to_string(),
            data: value.data,
            value: value.value,
            gas: value.gas,
            gas_price: value.gas_price,
            chain_id: value.chain_id,
            nonce: value.nonce,
        }
    }
}

/// Complete tool-facing transaction plan including both quote context and calldata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionPlan {
    pub quote: QuoteSummary,
    pub transaction: TransactionSummary,
}

impl OdosClient {
    /// Quote a single-token swap using the generic tooling request shape.
    pub async fn quote_for_tooling(&self, request: &SwapRequest) -> Result<QuoteSummary> {
        let request = request.validate()?;
        let quote = self.quote(&request.quote_request()).await?;
        Ok(QuoteSummary::from_quote(&request, &quote))
    }

    /// Build a transaction plan for a single-token swap using the generic
    /// tooling request shape.
    pub async fn build_transaction_plan(&self, request: &SwapRequest) -> Result<TransactionPlan> {
        let request = request.validate()?;
        let quote = self.quote(&request.quote_request()).await?;
        let tx = self
            .assemble_tx_data(request.signer, request.recipient, quote.path_id())
            .await?;

        Ok(TransactionPlan {
            quote: QuoteSummary::from_quote(&request, &quote),
            transaction: tx.into(),
        })
    }
}

fn parse_address(field: &str, value: &str) -> Result<Address> {
    value.parse().map_err(|err| {
        crate::OdosError::invalid_input(format!(
            "{field} must be a valid 0x-prefixed EVM address: {err}"
        ))
    })
}

fn parse_amount(field: &str, value: &str) -> Result<U256> {
    parse_value(value).map_err(|err| {
        crate::OdosError::invalid_input(format!(
            "{field} must be a decimal or hexadecimal integer amount: {err}"
        ))
    })
}

fn resolve_slippage(percent: Option<f64>, bps: Option<u16>) -> Result<Slippage> {
    match (percent, bps) {
        (Some(percent), Some(bps)) => {
            let percent_slippage =
                Slippage::percent(percent).map_err(crate::OdosError::invalid_input)?;
            let bps_slippage = Slippage::bps(bps).map_err(crate::OdosError::invalid_input)?;

            if percent_slippage.as_bps() != bps_slippage.as_bps() {
                return Err(crate::OdosError::invalid_input(format!(
                    "slippagePercent ({percent}) and slippageBps ({bps}) disagree"
                )));
            }

            Ok(percent_slippage)
        }
        (Some(percent), None) => {
            Slippage::percent(percent).map_err(crate::OdosError::invalid_input)
        }
        (None, Some(bps)) => Slippage::bps(bps).map_err(crate::OdosError::invalid_input),
        (None, None) => Ok(Slippage::standard()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;

    #[test]
    fn test_swap_request_defaults() {
        let request = SwapRequest {
            chain: ChainInput::Name("base".to_string()),
            from_token: "0x4200000000000000000000000000000000000006".to_string(),
            from_amount: "1000000000000000".to_string(),
            to_token: "0x833589fCD6EDb6E08f4c7C32D4f71b54bdA02913".to_string(),
            signer: "0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0".to_string(),
            recipient: None,
            slippage_percent: None,
            slippage_bps: None,
            referral_code: None,
            compact: None,
            simple: None,
            disable_rfqs: None,
        };

        let validated = request.validate().unwrap();
        assert_eq!(validated.chain, Chain::base());
        assert_eq!(validated.recipient, validated.signer);
        assert_eq!(validated.slippage, Slippage::standard());
        assert_eq!(validated.referral, ReferralCode::NONE);
        assert!(!validated.compact);
        assert!(!validated.simple);
        assert!(!validated.disable_rfqs);
    }

    #[test]
    fn test_swap_request_rejects_same_token() {
        let request = SwapRequest {
            chain: ChainInput::Id(1),
            from_token: address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").to_string(),
            from_amount: "1000000".to_string(),
            to_token: address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").to_string(),
            signer: address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0").to_string(),
            recipient: None,
            slippage_percent: Some(0.5),
            slippage_bps: None,
            referral_code: None,
            compact: None,
            simple: None,
            disable_rfqs: None,
        };

        let err = request.validate().unwrap_err();
        assert!(err.to_string().contains("must be different"));
    }

    #[test]
    fn test_swap_request_accepts_matching_slippage_inputs() {
        let request = SwapRequest {
            chain: ChainInput::Name("ethereum".to_string()),
            from_token: address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").to_string(),
            from_amount: "1000000".to_string(),
            to_token: address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").to_string(),
            signer: address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0").to_string(),
            recipient: None,
            slippage_percent: Some(0.5),
            slippage_bps: Some(50),
            referral_code: Some(42),
            compact: Some(true),
            simple: Some(false),
            disable_rfqs: Some(true),
        };

        let validated = request.validate().unwrap();
        assert_eq!(validated.slippage.as_bps(), 50);
        assert_eq!(validated.referral.code(), 42);
        assert!(validated.compact);
        assert!(validated.disable_rfqs);
    }

    #[test]
    fn test_swap_request_rejects_conflicting_slippage_inputs() {
        let request = SwapRequest {
            chain: ChainInput::Name("ethereum".to_string()),
            from_token: address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").to_string(),
            from_amount: "1000000".to_string(),
            to_token: address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").to_string(),
            signer: address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0").to_string(),
            recipient: None,
            slippage_percent: Some(0.5),
            slippage_bps: Some(75),
            referral_code: None,
            compact: None,
            simple: None,
            disable_rfqs: None,
        };

        let err = request.validate().unwrap_err();
        assert!(err.to_string().contains("disagree"));
    }
}
