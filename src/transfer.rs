use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};
use bon::Builder;

/// A transfer of a token from one address to another.
#[derive(Builder, Debug, Clone, Copy)]
pub struct TransferRouterFunds {
    chain: NamedChain,
    from: Address,
    to: Address,
    token: Address,
    amount: U256,
}

impl TransferRouterFunds {
    /// Get the chain of the transfer.
    pub fn chain(&self) -> NamedChain {
        self.chain
    }

    /// Get the sender of the transfer.
    pub fn from(&self) -> Address {
        self.from
    }

    /// Get the recipient of the transfer.
    pub fn to(&self) -> Address {
        self.to
    }

    /// Get the token that is being transferred.
    pub fn token(&self) -> Address {
        self.token
    }

    /// Get the amount of the token that is being transferred.
    pub fn amount(&self) -> U256 {
        self.amount
    }

    /// Get the parameters for the `transferRouterFunds` function.
    pub fn transfer_router_funds_params(&self) -> (Vec<Address>, Vec<U256>, Address) {
        // tokens, amounts, output_recipient
        (vec![self.token], vec![self.amount], self.to)
    }
}
