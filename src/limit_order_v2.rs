// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

use std::{fmt::Debug, marker::PhantomData};

use alloy_contract::CallBuilder;
use alloy_network::Ethereum;
use alloy_primitives::{Address, U256};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_sol_types::sol;
use OdosLimitOrderRouter::TokenInfo;
use OdosLimitOrderV2::OdosLimitOrderV2Instance;

use crate::SwapInputs;

/// The Limit Order Router contract.
#[derive(Debug, Clone)]
pub struct LimitOrderV2<P: Provider<Ethereum>> {
    instance: OdosLimitOrderV2Instance<P>,
}

impl<P: Provider<Ethereum>> LimitOrderV2<P> {
    pub fn new(address: Address, provider: P) -> Self {
        Self {
            instance: OdosLimitOrderV2Instance::new(address, provider),
        }
    }

    pub async fn liquidator_address(&self) -> Result<Address, alloy_contract::Error> {
        self.instance.liquidatorAddress().call().await
    }

    pub async fn owner(&self) -> Result<Address, alloy_contract::Error> {
        self.instance.owner().call().await
    }

    pub fn change_liquidator_address(
        &self,
        account: Address,
    ) -> CallBuilder<&P, PhantomData<OdosLimitOrderV2::changeLiquidatorAddressCall>> {
        self.instance.changeLiquidatorAddress(account)
    }

    pub fn build_swap_router_funds_call(
        &self,
        from: Address,
        to: Address,
        inputs: &SwapInputs,
    ) -> CallBuilder<&P, PhantomData<OdosLimitOrderV2::swapRouterFundsCall>> {
        self.instance
            .swapRouterFunds(
                vec![TokenInfo::from((
                    inputs.token_address(),
                    inputs.amount_in(),
                ))],
                vec![inputs.receiver()],
                TokenInfo::from((inputs.output_token_address(), inputs.value_out_min())),
                to,
                inputs.path_definition().clone(),
                inputs.executor(),
            )
            .from(from)
    }

    pub fn transfer_router_funds_request(
        &self,
        from: Address,
        token: Address,
        amount: U256,
        output_recipient: Address,
    ) -> TransactionRequest {
        self.transfer_router_funds(from, token, amount, output_recipient)
            .into_transaction_request()
    }

    pub fn transfer_router_funds(
        &self,
        from: Address,
        token: Address,
        amount: U256,
        output_recipient: Address,
    ) -> CallBuilder<&P, PhantomData<OdosLimitOrderV2::transferRouterFundsCall>> {
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

impl Debug for OdosLimitOrderV2::swapRouterFundsReturn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "amountOut: {}", self.amountOut)
    }
}

impl Debug for OdosLimitOrderRouter::TokenInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenInfo")
            .field("input_token", &self.tokenAddress)
            .field("input_amount", &self.tokenAmount)
            .finish()
    }
}

// codegen the odos_limit_order_v2 contract
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    OdosLimitOrderV2,
    "abis/odos_limit_order_v2.json"
);
