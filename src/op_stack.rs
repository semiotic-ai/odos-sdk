// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

//! OP-stack chain support for Odos SDK.
//!
//! This module provides specialized types for OP-stack chains (Optimism, Base, Mode,
//! Fraxtal) that give access to L1 gas information in transaction receipts.
//!
//! # L1 Gas Information
//!
//! When executing swaps on OP-stack chains, the total transaction cost includes both L2 execution
//! and L1 data availability costs. The OP-stack receipt types expose this information:
//!
//! ```rust,ignore
//! use odos_sdk::op_stack::OpTransactionReceipt;
//!
//! // After executing a swap on Base/Optimism
//! let receipt: OpTransactionReceipt = /* ... */;
//!
//! // Access L1 gas information
//! if let Some(l1_gas_used) = receipt.l1_gas_used {
//!     println!("L1 gas used: {l1_gas_used}");
//! }
//! if let Some(l1_fee) = receipt.l1_fee {
//!     println!("L1 fee: {l1_fee}");
//! }
//! if let Some(l1_gas_price) = receipt.l1_gas_price {
//!     println!("L1 gas price: {l1_gas_price}");
//! }
//! if let Some(l1_fee_scalar) = receipt.l1_fee_scalar {
//!     println!("L1 fee scalar: {l1_fee_scalar}");
//! }
//! ```
//!
//! # Supported OP-stack Chains
//!
//! - **Optimism** (chain ID 10)
//! - **Base** (chain ID 8453)
//! - **Mode** (chain ID 34443)
//! - **Fraxtal** (chain ID 252)
//!
//! # Usage with Standard Routers
//!
//! The standard [`V2Router`](crate::V2Router) and [`V3Router`](crate::V3Router) work on OP-stack
//! chains. To access L1 gas information, create a provider with the `Optimism` network type:
//!
//! ```rust,ignore
//! use odos_sdk::op_stack::Optimism;
//! use alloy_provider::ProviderBuilder;
//!
//! // Create a provider for OP-stack chains
//! let provider = ProviderBuilder::new()
//!     .network::<Optimism>()
//!     .connect_http("https://mainnet.base.org".parse()?);
//!
//! // Get transaction receipt with L1 gas info
//! let receipt = provider.get_transaction_receipt(tx_hash).await?;
//! if let Some(l1_fee) = receipt.inner.l1_fee {
//!     println!("L1 fee paid: {l1_fee}");
//! }
//! ```

pub use op_alloy_network::Optimism;
pub use op_alloy_rpc_types::OpTransactionReceipt;

/// Checks if a chain ID is an OP-stack chain supported by Odos.
///
/// Returns `true` for Optimism (10), Base (8453), Mode (34443), and Fraxtal (252).
pub const fn is_op_stack_chain(chain_id: u64) -> bool {
    matches!(chain_id, 10 | 8453 | 34443 | 252)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_op_stack_chain_detection() {
        // OP-stack chains
        assert!(is_op_stack_chain(10)); // Optimism
        assert!(is_op_stack_chain(8453)); // Base
        assert!(is_op_stack_chain(34443)); // Mode
        assert!(is_op_stack_chain(252)); // Fraxtal

        // Non-OP-stack chains
        assert!(!is_op_stack_chain(1)); // Ethereum
        assert!(!is_op_stack_chain(42161)); // Arbitrum
        assert!(!is_op_stack_chain(137)); // Polygon
    }
}
