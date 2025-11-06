use std::fmt::Display;

use alloy_network::TransactionBuilder;
use alloy_primitives::{hex, Address, U256};
use alloy_rpc_types::TransactionRequest;
use serde::{Deserialize, Serialize};

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
    type Error = crate::OdosError;

    fn try_from(data: TransactionData) -> Result<Self, Self::Error> {
        let input = hex::decode(&data.data)?;
        let value = parse_value(&data.value)?;

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

/// Parse a value string as U256, supporting both decimal and hexadecimal formats
///
/// This function attempts to parse the value as decimal first, then as hexadecimal
/// (with optional "0x" prefix) if decimal parsing fails.
///
/// # Arguments
///
/// * `value` - The string value to parse
///
/// # Returns
///
/// * `Ok(U256)` - The parsed value
/// * `Err(OdosError)` - If the value cannot be parsed in either format
///
/// # Examples
///
/// ```rust
/// # use odos_sdk::parse_value;
/// # use alloy_primitives::U256;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Decimal format
/// let val = parse_value("1000")?;
/// assert_eq!(val, U256::from(1000));
///
/// // Hexadecimal with 0x prefix
/// let val = parse_value("0xff")?;
/// assert_eq!(val, U256::from(255));
///
/// // Hexadecimal without prefix
/// let val = parse_value("ff")?;
/// assert_eq!(val, U256::from(255));
/// # Ok(())
/// # }
/// ```
pub fn parse_value(value: &str) -> crate::Result<U256> {
    use crate::OdosError;

    if value == "0" {
        return Ok(U256::ZERO);
    }

    // Try parsing as decimal first
    U256::from_str_radix(value, 10).or_else(|decimal_err| {
        // If decimal fails, try hexadecimal (with optional "0x" prefix)
        let hex_value = value.strip_prefix("0x").unwrap_or(value);
        U256::from_str_radix(hex_value, 16).map_err(|hex_err| {
            OdosError::invalid_input(format!(
                "Failed to parse value '{}' as decimal ({}) or hexadecimal ({})",
                value, decimal_err, hex_err
            ))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_value_zero() {
        let result = parse_value("0").unwrap();
        assert_eq!(result, U256::ZERO);
    }

    #[test]
    fn test_parse_value_decimal() {
        // Small values
        assert_eq!(parse_value("1").unwrap(), U256::from(1));
        assert_eq!(parse_value("123").unwrap(), U256::from(123));
        assert_eq!(parse_value("1000").unwrap(), U256::from(1000));

        // Large values
        assert_eq!(
            parse_value("1000000000000000000").unwrap(),
            U256::from(1000000000000000000u64)
        );

        // Very large values (beyond u64)
        let large_decimal = "123456789012345678901234567890";
        let result = parse_value(large_decimal);
        assert!(result.is_ok(), "Should parse large decimal values");
    }

    #[test]
    fn test_parse_value_hex_with_prefix() {
        // With 0x prefix
        assert_eq!(parse_value("0x0").unwrap(), U256::ZERO);
        assert_eq!(parse_value("0xff").unwrap(), U256::from(255));
        assert_eq!(parse_value("0xFF").unwrap(), U256::from(255));
        assert_eq!(parse_value("0x1234").unwrap(), U256::from(0x1234));
        assert_eq!(parse_value("0xabcdef").unwrap(), U256::from(0xabcdef));
    }

    #[test]
    fn test_parse_value_hex_without_prefix() {
        // Pure hex letters (no decimal interpretation) - falls back to hex parsing
        assert_eq!(parse_value("ff").unwrap(), U256::from(255));
        assert_eq!(parse_value("FF").unwrap(), U256::from(255));
        assert_eq!(parse_value("abcdef").unwrap(), U256::from(0xabcdef));
        assert_eq!(parse_value("ABCDEF").unwrap(), U256::from(0xabcdef));

        // Ambiguous: "1234" can be decimal or hex
        // Decimal parsing takes precedence, so this is 1234 not 0x1234
        assert_eq!(parse_value("1234").unwrap(), U256::from(1234));
        assert_ne!(parse_value("1234").unwrap(), U256::from(0x1234));
    }

    #[test]
    fn test_parse_value_invalid() {
        // Invalid characters (not valid decimal or hex)
        let result = parse_value("xyz");
        assert!(result.is_err(), "Invalid characters should fail");

        // Mixed invalid
        let result = parse_value("0xGHI");
        assert!(result.is_err(), "Invalid hex characters should fail");

        // Special characters
        let result = parse_value("12@34");
        assert!(result.is_err(), "Special characters should fail");

        // Note: Empty string "" actually succeeds with from_str_radix(10) -> returns 0
        // This is standard Rust behavior, so we accept it
        let result = parse_value("");
        assert_eq!(
            result.unwrap(),
            U256::ZERO,
            "Empty string parses to zero (standard Rust behavior)"
        );
    }

    #[test]
    fn test_parse_value_edge_cases() {
        // Leading zeros
        assert_eq!(parse_value("00123").unwrap(), U256::from(123));
        assert_eq!(parse_value("0x00ff").unwrap(), U256::from(255));

        // Max u64
        let max_u64_str = u64::MAX.to_string();
        let result = parse_value(&max_u64_str).unwrap();
        assert_eq!(result, U256::from(u64::MAX));

        // Max u128
        let max_u128_str = u128::MAX.to_string();
        let result = parse_value(&max_u128_str);
        assert!(result.is_ok(), "Should handle u128::MAX");
    }

    #[test]
    fn test_parse_value_realistic_transaction_values() {
        // 1 ETH in wei
        let one_eth = "1000000000000000000";
        assert_eq!(
            parse_value(one_eth).unwrap(),
            U256::from(1000000000000000000u64)
        );

        // 100 ETH in wei (typical transaction)
        let hundred_eth = "100000000000000000000";
        let result = parse_value(hundred_eth);
        assert!(result.is_ok(), "Should parse 100 ETH");

        // Gas price in hex (common format)
        let gas_price_hex = "0x2540be400"; // 10 gwei
        let result = parse_value(gas_price_hex);
        assert!(result.is_ok(), "Should parse hex gas price");
    }

    #[test]
    fn test_parse_value_error_messages() {
        // Verify error messages contain useful info
        let result = parse_value("invalid");
        match result {
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("invalid"),
                    "Error should mention the invalid value"
                );
                assert!(
                    error_msg.contains("decimal") || error_msg.contains("hexadecimal"),
                    "Error should mention attempted parsing formats"
                );
            }
            Ok(_) => panic!("Should have failed to parse 'invalid'"),
        }
    }
}
