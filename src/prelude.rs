// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! Prelude module for convenient imports
//!
//! This module re-exports the most commonly used types, traits, and functions
//! from the Odos SDK, allowing users to import everything they need with a single use statement.
//!
//! # Examples
//!
//! ```rust
//! use odos_sdk::prelude::*;
//! use alloy_primitives::{Address, U256};
//! use std::str::FromStr;
//!
//! # async fn example() -> Result<()> {
//! // Create a client
//! let client = OdosClient::new()?;
//!
//! // Build a quote request using the high-level SwapBuilder API
//! let usdc = Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?;
//! let weth = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")?;
//!
//! let quote = client.swap()
//!     .chain(Chain::ethereum())
//!     .from_token(usdc, U256::from(1_000_000)) // 1 USDC
//!     .to_token(weth)
//!     .slippage(Slippage::percent(0.5).unwrap())
//!     .signer(Address::from_str("0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0")?)
//!     .quote()
//!     .await?;
//!
//! println!("Expected output: {} WETH", quote.out_amount().unwrap_or(&"0".to_string()));
//! # Ok(())
//! # }
//! ```

// Re-export the most commonly used types

// Client
pub use crate::OdosClient;

// Request and response types
pub use crate::{AssemblyRequest, AssemblyResponse, QuoteRequest, SingleQuoteResponse};

// High-level builder API
pub use crate::SwapBuilder;

// Type-safe domain types
#[cfg(feature = "v2")]
pub use crate::{Chain, ReferralCode, Slippage};

// Error types
pub use crate::{OdosError, Result};

// Configuration
pub use crate::{ClientConfig, Endpoint, RetryConfig};

// Chain support trait
pub use crate::OdosChain;

// Common alloy re-exports for convenience
pub use alloy_primitives::{Address, U256};
