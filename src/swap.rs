use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};
use bon::Builder;

/// A transfer of a token from one address to another.
#[derive(Builder, Debug, Clone)]
pub struct Swap {
    chain: NamedChain,
    router_address: Address,
    signer_address: Address,
    output_recipient: Address,
    token_address: Address,
    token_amount: U256,
    path_id: String,
}

impl Swap {
    pub fn chain(&self) -> NamedChain {
        self.chain
    }

    pub fn output_recipient(&self) -> Address {
        self.output_recipient
    }

    pub fn router_address(&self) -> Address {
        self.router_address
    }

    pub fn signer_address(&self) -> Address {
        self.signer_address
    }

    pub fn token_address(&self) -> Address {
        self.token_address
    }

    pub fn token_amount(&self) -> U256 {
        self.token_amount
    }

    pub fn path_id(&self) -> &str {
        &self.path_id
    }
}
