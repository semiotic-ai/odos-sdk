// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

use std::{fmt::Debug, marker::PhantomData};

use alloy_contract::CallBuilder;
use alloy_network::{Ethereum, Network};
use alloy_primitives::{Address, Bytes, U256};
use alloy_provider::Provider;
use alloy_sol_types::{sol, SolInterface};

use crate::SwapInputs;

// Import generated types after sol! macro
use IOdosRouterV3::{inputTokenInfo, outputTokenInfo, swapReferralInfo, swapTokenInfo};
use OdosV3Router::{swapCall, OdosV3RouterCalls, OdosV3RouterInstance, Swap, SwapMulti};

/// The V3 SOR Router contract.
///
/// This router is generic over the network type, allowing it to work with both standard
/// Ethereum networks and OP-stack networks (Optimism, Base, Mode, Fraxtal).
///
/// # Type Parameters
///
/// - `N`: The network type (defaults to `Ethereum`). Use `op_alloy_network::Optimism` for OP-stack chains.
/// - `P`: The provider type.
///
/// # Example
///
/// ```rust,ignore
/// use odos_sdk::V3Router;
/// use alloy_network::Ethereum;
/// use alloy_provider::ProviderBuilder;
///
/// // Standard Ethereum usage
/// let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse()?);
/// let router: V3Router<Ethereum, _> = V3Router::new(address, provider);
///
/// // OP-stack usage (requires op-stack feature)
/// #[cfg(feature = "op-stack")]
/// {
///     use odos_sdk::op_stack::Optimism;
///     let op_provider = ProviderBuilder::new()
///         .network::<Optimism>()
///         .connect_http("https://mainnet.base.org".parse()?);
///     let op_router: V3Router<Optimism, _> = V3Router::new(address, op_provider);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct V3Router<N: Network = Ethereum, P: Provider<N> = alloy_provider::RootProvider<N>> {
    instance: OdosV3RouterInstance<P, N>,
}

impl<N: Network, P: Provider<N>> V3Router<N, P> {
    /// Creates a new V3 router instance.
    pub fn new(address: Address, provider: P) -> Self {
        Self {
            instance: OdosV3RouterInstance::new(address, provider),
        }
    }

    /// Returns the contract owner address.
    pub async fn owner(&self) -> Result<Address, alloy_contract::Error> {
        self.instance.owner().call().await
    }

    /// Returns the liquidator address.
    pub async fn liquidator_address(&self) -> Result<Address, alloy_contract::Error> {
        self.instance.liquidatorAddress().call().await
    }

    /// Builds a swap call using router funds.
    pub fn build_swap_router_funds_call(
        &self,
        input_token_info: inputTokenInfo,
        output_token_info: outputTokenInfo,
        inputs: &SwapInputs,
        from: Address,
    ) -> CallBuilder<&P, PhantomData<OdosV3Router::swapRouterFundsCall>, N> {
        self.instance
            .swapRouterFunds(
                vec![input_token_info],
                vec![output_token_info],
                inputs.path_definition().clone(),
                inputs.executor(),
            )
            .from(from)
    }

    /// Transfers router funds to a recipient.
    pub fn transfer_router_funds(
        &self,
        from: Address,
        token: Address,
        amount: U256,
        output_recipient: Address,
    ) -> CallBuilder<&P, PhantomData<OdosV3Router::transferRouterFundsCall>, N> {
        self.instance
            .transferRouterFunds(vec![token], vec![amount], output_recipient)
            .from(from)
    }

    /// Returns the calldata for a transfer router funds call.
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
