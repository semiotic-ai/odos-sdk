// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};
use bon::Builder;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A transfer of a token from one address to another.
#[derive(Builder, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
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

impl Serialize for TransferRouterFunds {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let chain_id: u64 = self.chain.into();
        let data = (chain_id, self.from, self.to, self.token, self.amount);
        data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TransferRouterFunds {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (chain_id, from, to, token, amount): (u64, Address, Address, Address, U256) =
            Deserialize::deserialize(deserializer)?;
        let chain = NamedChain::try_from(chain_id).map_err(serde::de::Error::custom)?;

        Ok(Self {
            chain,
            from,
            to,
            token,
            amount,
        })
    }
}
