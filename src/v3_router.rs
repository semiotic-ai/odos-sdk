// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

use std::{fmt::Debug, marker::PhantomData};

use alloy_contract::CallBuilder;
use alloy_network::Ethereum;
use alloy_primitives::{Address, Bytes, U256};
use alloy_provider::Provider;
use alloy_sol_types::{sol, SolInterface};

use crate::SwapInputs;

// Import generated types after sol! macro
use IOdosRouterV3::{inputTokenInfo, outputTokenInfo, swapReferralInfo, swapTokenInfo};
use OdosV3Router::{swapCall, OdosV3RouterCalls, OdosV3RouterInstance, Swap, SwapMulti};

/// The V2 SOR Router contract.
#[derive(Debug, Clone)]
pub struct V3Router<P: Provider<Ethereum>> {
    instance: OdosV3RouterInstance<P>,
}

impl<P: Provider<Ethereum>> V3Router<P> {
    pub fn new(address: Address, provider: P) -> Self {
        Self {
            instance: OdosV3RouterInstance::new(address, provider),
        }
    }

    pub async fn owner(&self) -> Result<Address, alloy_contract::Error> {
        self.instance.owner().call().await
    }

    pub async fn liquidator_address(&self) -> Result<Address, alloy_contract::Error> {
        self.instance.liquidatorAddress().call().await
    }

    pub fn build_swap_router_funds_call(
        &self,
        input_token_info: inputTokenInfo,
        output_token_info: outputTokenInfo,
        inputs: &SwapInputs,
        from: Address,
    ) -> CallBuilder<&P, PhantomData<OdosV3Router::swapRouterFundsCall>> {
        self.instance
            .swapRouterFunds(
                vec![input_token_info],
                vec![output_token_info],
                inputs.path_definition().clone(),
                inputs.executor(),
            )
            .from(from)
    }

    pub fn transfer_router_funds(
        &self,
        from: Address,
        token: Address,
        amount: U256,
        output_recipient: Address,
    ) -> CallBuilder<&P, PhantomData<OdosV3Router::transferRouterFundsCall>> {
        self.instance
            .transferRouterFunds(vec![token], vec![amount], output_recipient)
            .from(from)
    }

    pub fn transfer_router_funds_calldata(
        &self,
        from: Address,
        token: Address,
        amount: U256,
        output_recipient: Address,
    ) -> Vec<u8> {
        self.transfer_router_funds(from, token, amount, output_recipient)
            .calldata()
            .to_vec()
    }
}

// codegen the odos_v3_router contract
sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    OdosV3Router,
    "abis/v3.json"
);

impl Debug for OdosV3Router::swapRouterFundsReturn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "amountsOut: {:?}", self.amountsOut)
    }
}

impl Debug for inputTokenInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("inputTokenInfo")
            .field("tokenAddress", &self.tokenAddress)
            .field("amountIn", &self.amountIn)
            .field("receiver", &self.receiver)
            .finish()
    }
}

impl Debug for outputTokenInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("outputTokenInfo")
            .field("tokenAddress", &self.tokenAddress)
            .field("amountQuote", &self.amountQuote)
            .field("amountMin", &self.amountMin)
            .field("receiver", &self.receiver)
            .finish()
    }
}

impl Debug for swapCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("swapCall")
            .field("executor", &self.executor)
            .field("pathDefinition", &self.pathDefinition)
            .field("referralInfo", &self.referralInfo)
            .field("tokenInfo", &self.tokenInfo)
            .finish()
    }
}

impl Debug for swapReferralInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("swapReferralInfo")
            .field("code", &self.code)
            .field("fee", &self.fee)
            .field("feeRecipient", &self.feeRecipient)
            .finish()
    }
}

impl Debug for SwapMulti {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwapMulti")
            .field("sender", &self.sender)
            .field("amountsIn", &self.amountsIn)
            .field("tokensIn", &self.tokensIn)
            .field("amountsOut", &self.amountsOut)
            .field("tokensOut", &self.tokensOut)
            .finish()
    }
}

impl Debug for Swap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Swap")
            .field("sender", &self.sender)
            .field("inputAmount", &self.inputAmount)
            .field("inputToken", &self.inputToken)
            .field("amountOut", &self.amountOut)
            .field("outputToken", &self.outputToken)
            .field("slippage", &self.slippage)
            .field("referralCode", &self.referralCode)
            .finish()
    }
}

impl Debug for swapTokenInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("swapTokenInfo")
            .field("inputToken", &self.inputToken)
            .field("inputAmount", &self.inputAmount)
            .field("inputReceiver", &self.inputReceiver)
            .field("outputToken", &self.outputToken)
            .field("outputQuote", &self.outputQuote)
            .field("outputMin", &self.outputMin)
            .field("outputReceiver", &self.outputReceiver)
            .finish()
    }
}

impl TryFrom<&Bytes> for OdosV3RouterCalls {
    type Error = alloy_sol_types::Error;

    fn try_from(input: &Bytes) -> Result<Self, Self::Error> {
        OdosV3RouterCalls::abi_decode(input)
    }
}
