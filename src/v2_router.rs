use std::{fmt::Debug, marker::PhantomData};

use alloy_contract::CallBuilder;
use alloy_network::Ethereum;
use alloy_primitives::{Address, Bytes, U256};
use alloy_provider::Provider;
use alloy_sol_types::{sol, SolInterface};

use crate::SwapInputs;

// Import generated types after sol! macro
use OdosRouterV2::{inputTokenInfo, outputTokenInfo, swapTokenInfo};
use OdosV2Router::{swapCall, OdosV2RouterCalls, OdosV2RouterInstance, Swap, SwapMulti};

/// The V2 SOR Router contract.
#[derive(Debug, Clone)]
pub struct V2Router<P: Provider<Ethereum>> {
    instance: OdosV2RouterInstance<P>,
}

impl<P: Provider<Ethereum>> V2Router<P> {
    pub fn new(address: Address, provider: P) -> Self {
        Self {
            instance: OdosV2RouterInstance::new(address, provider),
        }
    }

    pub async fn owner(&self) -> Result<Address, alloy_contract::Error> {
        self.instance.owner().call().await
    }

    pub fn build_swap_router_funds_call(
        &self,
        input_token_info: inputTokenInfo,
        output_token_info: outputTokenInfo,
        inputs: &SwapInputs,
        from: Address,
    ) -> CallBuilder<&P, PhantomData<OdosV2Router::swapRouterFundsCall>> {
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

    pub fn transfer_router_funds(
        &self,
        from: Address,
        token: Address,
        amount: U256,
        output_recipient: Address,
    ) -> CallBuilder<&P, PhantomData<OdosV2Router::transferRouterFundsCall>> {
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
