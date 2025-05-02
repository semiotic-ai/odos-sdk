use std::fmt::Display;

use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};
use bon::Builder;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A token swap.
#[derive(Builder, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SwapContext {
    /// The chain of the swap.
    chain: NamedChain,
    /// The address of the router.
    router_address: Address,
    /// The address of the signer.
    signer_address: Address,
    /// The address of the recipient of the output token.
    output_recipient: Address,
    /// The address of the token to swap.
    token_address: Address,
    /// The amount of tokens to swap.
    token_amount: U256,
    /// The path ID of the swap.
    path_id: String,
}

impl SwapContext {
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

impl Serialize for SwapContext {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let chain_id: u64 = self.chain.into();
        let data = (
            chain_id,
            self.router_address,
            self.signer_address,
            self.output_recipient,
            self.token_address,
            self.token_amount,
            &self.path_id,
        );
        data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SwapContext {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (
            chain_id,
            router_address,
            signer_address,
            output_recipient,
            token_address,
            token_amount,
            path_id,
        ): (u64, Address, Address, Address, Address, U256, String) =
            Deserialize::deserialize(deserializer)?;

        let chain = NamedChain::try_from(chain_id).map_err(serde::de::Error::custom)?;

        Ok(Self {
            chain,
            router_address,
            signer_address,
            output_recipient,
            token_address,
            token_amount,
            path_id,
        })
    }
}

impl Display for SwapContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Swap {{ chain: {}, router_address: {}, signer_address: {}, output_recipient: {}, token_address: {}, token_amount: {}, path_id: {} }}",
            self.chain,
            self.router_address,
            self.signer_address,
            self.output_recipient,
            self.token_address,
            self.token_amount,
            self.path_id
        )
    }
}
