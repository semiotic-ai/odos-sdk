use alloy_network::Ethereum;
use alloy_primitives::Address;
use alloy_primitives::U256;
use alloy_provider::PendingTransactionBuilder;
use alloy_provider::Provider;
use alloy_sol_types::sol;

use crate::ERC20::ERC20Instance;

/// The ERC20 contract.
#[derive(Debug, Clone)]
pub struct Erc20<P: Provider<Ethereum>> {
    instance: ERC20Instance<P>,
}

impl<P: Provider<Ethereum>> Erc20<P> {
    pub fn new(address: Address, provider: P) -> Self {
        Self {
            instance: ERC20Instance::new(address, provider),
        }
    }

    pub async fn balance_of(&self, address: Address) -> Result<U256, alloy_contract::Error> {
        self.instance.balanceOf(address).call().await
    }

    pub async fn allowance(
        &self,
        owner: Address,
        spender: Address,
    ) -> Result<U256, alloy_contract::Error> {
        self.instance.allowance(owner, spender).call().await
    }

    pub async fn decimals(&self) -> Result<u8, alloy_contract::Error> {
        self.instance.decimals().call().await
    }

    pub async fn approve(
        &self,
        owner: Address,
        spender: Address,
        amount: U256,
    ) -> Result<PendingTransactionBuilder<Ethereum>, alloy_contract::Error> {
        self.instance
            .approve(spender, amount)
            .from(owner)
            .send()
            .await
    }

    pub async fn transfer(
        &self,
        from: Address,
        to: Address,
        amount: U256,
    ) -> Result<PendingTransactionBuilder<Ethereum>, alloy_contract::Error> {
        self.instance.transfer(to, amount).from(from).send().await
    }
}

// Codegen from abi.
sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "abis/erc20_abi.json"
);
