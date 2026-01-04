// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

//! Utilities for batching on-chain queries.
//!
//! This module provides two approaches for checking token balances and allowances:
//!
//! ## Parallel RPC (Recommended for 2-5 calls)
//!
//! Uses `tokio::join_all` to make parallel RPC calls. Simple, no dependencies,
//! works on any chain.
//!
//! ```rust,ignore
//! use odos_sdk::multicall::{check_balance, check_allowance};
//!
//! // Simple parallel checks
//! let (balance, allowance) = tokio::join!(
//!     check_balance(&provider, token, owner),
//!     check_allowance(&provider, token, owner, spender),
//! );
//! ```
//!
//! ## Multicall3 (Recommended for 10+ calls)
//!
//! Batches all calls into a single RPC request. More efficient at scale,
//! atomic state reads, but requires Multicall3 contract.
//!
//! ```rust,ignore
//! use odos_sdk::multicall::{multicall_check_balances, multicall_check_allowances};
//!
//! // Batch many tokens in one RPC call
//! let tokens = vec![usdc, weth, dai, /* ... many more */];
//! let balances = multicall_check_balances(&provider, owner, &tokens).await?;
//! ```
//!
//! ## When to Use Which
//!
//! | Calls | Recommendation | Reason |
//! |-------|---------------|--------|
//! | 1-5   | Parallel RPC  | Simpler, no contract dependency |
//! | 5-10  | Either        | Similar performance |
//! | 10+   | Multicall3    | Single RPC, lower latency, less rate limiting |

use alloy_network::{Ethereum, Network};
use alloy_primitives::{Address, U256};
use alloy_provider::Provider;
use alloy_rpc_types::TransactionRequest;
use alloy_sol_types::{sol, SolCall};

// =============================================================================
// Simple Parallel RPC Functions (no contract dependency)
// =============================================================================

/// Checks the ERC20 balance of an address.
///
/// Makes a single `eth_call` to the token contract.
///
/// # Example
///
/// ```rust,ignore
/// let balance = check_balance(&provider, usdc_address, my_address).await?;
/// ```
pub async fn check_balance<P>(
    provider: &P,
    token: Address,
    owner: Address,
) -> Result<U256, alloy_transport::TransportError>
where
    P: Provider<Ethereum>,
{
    let calldata = balanceOfCall { owner }.abi_encode();
    let tx = TransactionRequest::default()
        .to(token)
        .input(calldata.into());

    let result = provider.call(tx).await?;

    if result.len() >= 32 {
        Ok(U256::from_be_slice(&result[..32]))
    } else {
        Ok(U256::ZERO)
    }
}

/// Checks the ERC20 allowance for a spender.
///
/// Makes a single `eth_call` to the token contract.
///
/// # Example
///
/// ```rust,ignore
/// let allowance = check_allowance(&provider, usdc_address, my_address, router_address).await?;
/// ```
pub async fn check_allowance<P>(
    provider: &P,
    token: Address,
    owner: Address,
    spender: Address,
) -> Result<U256, alloy_transport::TransportError>
where
    P: Provider<Ethereum>,
{
    let calldata = allowanceCall { owner, spender }.abi_encode();
    let tx = TransactionRequest::default()
        .to(token)
        .input(calldata.into());

    let result = provider.call(tx).await?;

    if result.len() >= 32 {
        Ok(U256::from_be_slice(&result[..32]))
    } else {
        Ok(U256::ZERO)
    }
}

/// Checks balance and allowance in parallel using tokio.
///
/// This is the recommended approach for single-token pre-flight checks.
///
/// # Example
///
/// ```rust,ignore
/// let (balance, allowance) = check_balance_and_allowance(
///     &provider, usdc_address, my_address, router_address
/// ).await?;
///
/// if balance >= amount && allowance >= amount {
///     println!("Ready to swap!");
/// }
/// ```
pub async fn check_balance_and_allowance<P>(
    provider: &P,
    token: Address,
    owner: Address,
    spender: Address,
) -> Result<(U256, U256), alloy_transport::TransportError>
where
    P: Provider<Ethereum>,
{
    let balance_fut = check_balance(provider, token, owner);
    let allowance_fut = check_allowance(provider, token, owner, spender);

    let (balance, allowance) = tokio::join!(balance_fut, allowance_fut);

    Ok((balance?, allowance?))
}

// =============================================================================
// Multicall3 Functions (for batching many calls)
// =============================================================================

/// Multicall3 contract deployed at the same address on all major chains.
pub const MULTICALL3_ADDRESS: Address =
    alloy_primitives::address!("cA11bde05977b3631167028862bE2a173976CA11");

// ERC20 interface for balance and allowance calls
sol! {
    #[allow(missing_docs)]
    function balanceOf(address owner) external view returns (uint256);

    #[allow(missing_docs)]
    function allowance(address owner, address spender) external view returns (uint256);
}

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    interface IMulticall3 {
        struct Call3 {
            address target;
            bool allowFailure;
            bytes callData;
        }

        struct Result {
            bool success;
            bytes returnData;
        }

        function aggregate3(Call3[] calldata calls) external payable returns (Result[] memory returnData);
    }
}

use IMulticall3::{Call3, IMulticall3Instance, Result as MulticallResult};

/// A pre-flight check for swap prerequisites.
#[derive(Debug, Clone)]
pub struct SwapPreflightCheck {
    /// Token to check.
    pub token: Address,
    /// Owner address (who holds the tokens).
    pub owner: Address,
    /// Spender address (router that needs approval).
    pub spender: Address,
    /// Required amount for the swap.
    pub required_amount: U256,
}

/// Result of a pre-flight check.
#[derive(Debug, Clone)]
pub struct PreflightResult {
    /// The token checked.
    pub token: Address,
    /// Current balance of the owner.
    pub balance: U256,
    /// Current allowance for the spender.
    pub allowance: U256,
    /// Whether the balance is sufficient.
    pub sufficient_balance: bool,
    /// Whether the allowance is sufficient.
    pub sufficient_allowance: bool,
}

impl PreflightResult {
    /// Returns true if the swap can proceed (sufficient balance and allowance).
    pub fn is_ready(&self) -> bool {
        self.sufficient_balance && self.sufficient_allowance
    }

    /// Returns the amount of additional approval needed, or 0 if sufficient.
    pub fn approval_needed(&self, required: U256) -> U256 {
        if self.allowance >= required {
            U256::ZERO
        } else {
            required.saturating_sub(self.allowance)
        }
    }
}

/// Batch check ERC20 balances for multiple tokens using Multicall3.
///
/// Fetches all balances in a single RPC call. Recommended for 10+ tokens.
///
/// # Arguments
///
/// * `provider` - The Alloy provider
/// * `owner` - Address to check balances for
/// * `tokens` - List of token addresses to check
///
/// # Returns
///
/// A vector of balances corresponding to each token in the input list.
/// Failed calls return `U256::ZERO`.
///
/// # Example
///
/// ```rust,ignore
/// let tokens = vec![usdc, weth, dai, link, uni];
/// let balances = multicall_check_balances(&provider, my_address, &tokens).await?;
/// ```
pub async fn multicall_check_balances<N, P>(
    provider: &P,
    owner: Address,
    tokens: &[Address],
) -> Result<Vec<U256>, alloy_contract::Error>
where
    N: Network,
    P: Provider<N>,
{
    if tokens.is_empty() {
        return Ok(vec![]);
    }

    let multicall = IMulticall3Instance::new(MULTICALL3_ADDRESS, provider);

    let calls: Vec<Call3> = tokens
        .iter()
        .map(|&token| {
            let calldata = balanceOfCall { owner }.abi_encode();
            Call3 {
                target: token,
                allowFailure: true,
                callData: calldata.into(),
            }
        })
        .collect();

    let results: Vec<MulticallResult> = multicall.aggregate3(calls).call().await?;

    Ok(results
        .into_iter()
        .map(|result| {
            if result.success && result.returnData.len() >= 32 {
                U256::from_be_slice(&result.returnData[..32])
            } else {
                U256::ZERO
            }
        })
        .collect())
}

/// Batch check ERC20 allowances for multiple tokens using Multicall3.
///
/// Fetches all allowances in a single RPC call. Recommended for 10+ tokens.
///
/// # Arguments
///
/// * `provider` - The Alloy provider
/// * `owner` - Address that owns the tokens
/// * `spender` - Address to check allowance for (e.g., router address)
/// * `tokens` - List of token addresses to check
///
/// # Returns
///
/// A vector of allowances corresponding to each token in the input list.
/// Failed calls return `U256::ZERO`.
pub async fn multicall_check_allowances<N, P>(
    provider: &P,
    owner: Address,
    spender: Address,
    tokens: &[Address],
) -> Result<Vec<U256>, alloy_contract::Error>
where
    N: Network,
    P: Provider<N>,
{
    if tokens.is_empty() {
        return Ok(vec![]);
    }

    let multicall = IMulticall3Instance::new(MULTICALL3_ADDRESS, provider);

    let calls: Vec<Call3> = tokens
        .iter()
        .map(|&token| {
            let calldata = allowanceCall { owner, spender }.abi_encode();
            Call3 {
                target: token,
                allowFailure: true,
                callData: calldata.into(),
            }
        })
        .collect();

    let results: Vec<MulticallResult> = multicall.aggregate3(calls).call().await?;

    Ok(results
        .into_iter()
        .map(|result| {
            if result.success && result.returnData.len() >= 32 {
                U256::from_be_slice(&result.returnData[..32])
            } else {
                U256::ZERO
            }
        })
        .collect())
}

/// Perform pre-flight checks for multiple swaps using Multicall3.
///
/// Efficiently checks both balances and allowances in a single batched RPC call,
/// then returns detailed results for each token. Recommended for 10+ checks.
///
/// # Arguments
///
/// * `provider` - The Alloy provider
/// * `checks` - List of pre-flight checks to perform
///
/// # Returns
///
/// A vector of results indicating whether each swap is ready to execute.
pub async fn multicall_preflight_checks<N, P>(
    provider: &P,
    checks: &[SwapPreflightCheck],
) -> Result<Vec<PreflightResult>, alloy_contract::Error>
where
    N: Network,
    P: Provider<N>,
{
    if checks.is_empty() {
        return Ok(vec![]);
    }

    let multicall = IMulticall3Instance::new(MULTICALL3_ADDRESS, provider);

    // Build calls for both balances and allowances
    let mut calls: Vec<Call3> = Vec::with_capacity(checks.len() * 2);

    for check in checks {
        // Balance check
        let balance_calldata = balanceOfCall { owner: check.owner }.abi_encode();
        calls.push(Call3 {
            target: check.token,
            allowFailure: true,
            callData: balance_calldata.into(),
        });

        // Allowance check
        let allowance_calldata = allowanceCall {
            owner: check.owner,
            spender: check.spender,
        }
        .abi_encode();
        calls.push(Call3 {
            target: check.token,
            allowFailure: true,
            callData: allowance_calldata.into(),
        });
    }

    let results: Vec<MulticallResult> = multicall.aggregate3(calls).call().await?;

    // Parse results in pairs (balance, allowance)
    Ok(checks
        .iter()
        .enumerate()
        .map(|(i, check)| {
            let balance_result = &results[i * 2];
            let allowance_result = &results[i * 2 + 1];

            let balance = if balance_result.success && balance_result.returnData.len() >= 32 {
                U256::from_be_slice(&balance_result.returnData[..32])
            } else {
                U256::ZERO
            };

            let allowance = if allowance_result.success && allowance_result.returnData.len() >= 32 {
                U256::from_be_slice(&allowance_result.returnData[..32])
            } else {
                U256::ZERO
            };

            PreflightResult {
                token: check.token,
                balance,
                allowance,
                sufficient_balance: balance >= check.required_amount,
                sufficient_allowance: allowance >= check.required_amount,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;

    #[test]
    fn test_multicall3_address() {
        // Multicall3 is deployed at the same address on all major chains
        assert_eq!(
            MULTICALL3_ADDRESS,
            address!("cA11bde05977b3631167028862bE2a173976CA11")
        );
    }

    #[test]
    fn test_preflight_result_is_ready() {
        let result = PreflightResult {
            token: Address::ZERO,
            balance: U256::from(1000),
            allowance: U256::from(1000),
            sufficient_balance: true,
            sufficient_allowance: true,
        };
        assert!(result.is_ready());

        let result_insufficient = PreflightResult {
            token: Address::ZERO,
            balance: U256::from(1000),
            allowance: U256::from(100),
            sufficient_balance: true,
            sufficient_allowance: false,
        };
        assert!(!result_insufficient.is_ready());
    }

    #[test]
    fn test_approval_needed() {
        let result = PreflightResult {
            token: Address::ZERO,
            balance: U256::from(1000),
            allowance: U256::from(500),
            sufficient_balance: true,
            sufficient_allowance: false,
        };

        assert_eq!(result.approval_needed(U256::from(800)), U256::from(300));
        assert_eq!(result.approval_needed(U256::from(500)), U256::ZERO);
        assert_eq!(result.approval_needed(U256::from(300)), U256::ZERO);
    }

    #[test]
    fn test_balance_of_encoding() {
        let owner = address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0");
        let calldata = balanceOfCall { owner }.abi_encode();

        // balanceOf selector is 0x70a08231
        assert_eq!(&calldata[0..4], &[0x70, 0xa0, 0x82, 0x31]);
    }

    #[test]
    fn test_allowance_encoding() {
        let owner = address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0");
        let spender = address!("cf5540fffcdc3d510b18bfca6d2b9987b0772559");
        let calldata = allowanceCall { owner, spender }.abi_encode();

        // allowance selector is 0xdd62ed3e
        assert_eq!(&calldata[0..4], &[0xdd, 0x62, 0xed, 0x3e]);
    }
}
