use std::fmt::Display;

use alloy_network::TransactionBuilder;
use alloy_primitives::{hex, Address, U256};
use alloy_rpc_types::TransactionRequest;
use serde::{Deserialize, Serialize};

pub const ASSEMBLE_URL: &str = "https://api.odos.xyz/sor/assemble";

/// Request to the Odos Assemble API: <https://docs.odos.xyz/build/api-docs>
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssembleRequest {
    pub user_addr: String,
    pub path_id: String,
    pub simulate: bool,
    pub receiver: Option<Address>,
}

impl Display for AssembleRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AssembleRequest {{ user_addr: {}, path_id: {}, simulate: {}, receiver: {} }}",
            self.user_addr,
            self.path_id,
            self.simulate,
            self.receiver
                .as_ref()
                .map_or("None".to_string(), |s| s.to_string())
        )
    }
}

/// Response from the Odos Assemble API: <https://docs.odos.xyz/build/api-docs>
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssemblyResponse {
    pub transaction: TransactionData,
    pub simulation: Option<Simulation>,
}

impl Display for AssemblyResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AssemblyResponse {{ transaction: {}, simulation: {} }}",
            self.transaction,
            self.simulation
                .as_ref()
                .map_or("None".to_string(), |s| s.to_string())
        )
    }
}

/// Transaction data from the Odos Assemble API: <https://docs.odos.xyz/build/api-docs>
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionData {
    pub to: Address,
    pub from: Address,
    pub data: String,
    pub value: String,
    pub gas: i128,
    pub gas_price: u128,
    pub chain_id: u64,
    pub nonce: u64,
}

/// Convert [`TransactionData`] to a [`TransactionRequest`].
impl TryFrom<TransactionData> for TransactionRequest {
    type Error = hex::FromHexError;

    fn try_from(data: TransactionData) -> Result<Self, Self::Error> {
        let input = hex::decode(&data.data)?;
        let value = parse_value(&data.value);

        Ok(TransactionRequest::default()
            .with_input(input)
            .with_value(value))
    }
}

impl Display for TransactionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TransactionData {{ to: {}, from: {}, data: {}, value: {}, gas: {}, gas_price: {}, chain_id: {}, nonce: {} }}",
            self.to,
            self.from,
            self.data,
            self.value,
            self.gas,
            self.gas_price,
            self.chain_id,
            self.nonce
        )
    }
}

/// Simulation from the Odos Assemble API: <https://docs.odos.xyz/build/api-docs>
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Simulation {
    is_success: bool,
    amounts_out: Vec<String>,
    gas_estimate: i64,
    simulation_error: SimulationError,
}

impl Simulation {
    pub fn is_success(&self) -> bool {
        self.is_success
    }

    pub fn error_message(&self) -> &str {
        &self.simulation_error.error_message
    }
}

impl Display for Simulation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Simulation {{ is_success: {}, amounts_out: {:?}, gas_estimate: {}, simulation_error: {} }}",
            self.is_success,
            self.amounts_out,
            self.gas_estimate,
            self.simulation_error.error_message
        )
    }
}

/// Simulation error from the Odos Assemble API: <https://docs.odos.xyz/build/api-docs>
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationError {
    r#type: String,
    error_message: String,
}

impl SimulationError {
    pub fn error_message(&self) -> &str {
        &self.error_message
    }
}

impl Display for SimulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Simulation error: {}", self.error_message)
    }
}

pub fn parse_value(value: &str) -> U256 {
    if value == "0" {
        return U256::ZERO;
    }

    U256::from_str_radix(value, 10).unwrap_or_else(|_| {
        // Remove "0x" prefix if present
        let value = if let Some(value) = value.strip_prefix("0x") {
            value
        } else {
            value
        };
        U256::from_str_radix(value, 16).unwrap_or(U256::ZERO)
    })
}
