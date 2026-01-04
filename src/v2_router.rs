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
use OdosRouterV2::{inputTokenInfo, outputTokenInfo, swapTokenInfo};
use OdosV2Router::{swapCall, OdosV2RouterCalls, OdosV2RouterInstance, Swap, SwapMulti};

/// The V2 SOR Router contract.
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
/// use odos_sdk::V2Router;
/// use alloy_network::Ethereum;
/// use alloy_provider::ProviderBuilder;
///
/// // Standard Ethereum usage
/// let provider = ProviderBuilder::new().connect_http("https://eth.llamarpc.com".parse()?);
/// let router: V2Router<Ethereum, _> = V2Router::new(address, provider);
///
/// // OP-stack usage (requires op-stack feature)
/// #[cfg(feature = "op-stack")]
/// {
///     use odos_sdk::op_stack::Optimism;
///     let op_provider = ProviderBuilder::new()
///         .network::<Optimism>()
///         .connect_http("https://mainnet.base.org".parse()?);
///     let op_router: V2Router<Optimism, _> = V2Router::new(address, op_provider);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct V2Router<N: Network = Ethereum, P: Provider<N> = alloy_provider::RootProvider<N>> {
    instance: OdosV2RouterInstance<P, N>,
}

impl<N: Network, P: Provider<N>> V2Router<N, P> {
    /// Creates a new V2 router instance.
    pub fn new(address: Address, provider: P) -> Self {
        Self {
            instance: OdosV2RouterInstance::new(address, provider),
        }
    }

    /// Returns the contract owner address.
    pub async fn owner(&self) -> Result<Address, alloy_contract::Error> {
        self.instance.owner().call().await
    }

    /// Builds a swap call using router funds.
    pub fn build_swap_router_funds_call(
        &self,
        input_token_info: inputTokenInfo,
        output_token_info: outputTokenInfo,
        inputs: &SwapInputs,
        from: Address,
    ) -> CallBuilder<&P, PhantomData<OdosV2Router::swapRouterFundsCall>, N> {
        self.instance
            .swapRouterFunds(
                vec![input_token_info],
                vec![output_token_info],
                inputs.value_out_min(),
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
    ) -> CallBuilder<&P, PhantomData<OdosV2Router::transferRouterFundsCall>, N> {
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

// codegen the odos_v2_router contract
sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    OdosV2Router,
    "abis/odos_v2_router.json"
);

impl Debug for OdosV2Router::swapRouterFundsReturn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "amountsOut: {:?}", self.amountsOut)
    }
}

impl Debug for OdosRouterV2::inputTokenInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("inputTokenInfo")
            .field("tokenAddress", &self.tokenAddress)
            .field("amountIn", &self.amountIn)
            .field("receiver", &self.receiver)
            .finish()
    }
}

impl Debug for OdosRouterV2::outputTokenInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("outputTokenInfo")
            .field("tokenAddress", &self.tokenAddress)
            .field("relativeValue", &self.relativeValue)
            .field("receiver", &self.receiver)
            .finish()
    }
}

impl Debug for swapCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("swapCall")
            .field("executor", &self.executor)
            .field("pathDefinition", &self.pathDefinition)
            .field("referralCode", &self.referralCode)
            .field("tokenInfo", &self.tokenInfo)
            .finish()
    }
}

impl Debug for swapTokenInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("swapTokenInfo")
            .field("inputToken", &self.inputToken)
            .field("inputAmount", &self.inputAmount)
            .field("inputReceiver", &self.inputReceiver)
            .field("outputMin", &self.outputMin)
            .field("outputQuote", &self.outputQuote)
            .field("outputReceiver", &self.outputReceiver)
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

impl TryFrom<&Bytes> for OdosV2RouterCalls {
    type Error = alloy_sol_types::Error;

    fn try_from(input: &Bytes) -> Result<Self, Self::Error> {
        OdosV2RouterCalls::abi_decode(input)
    }
}
