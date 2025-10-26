//! # Odos SDK
//!
//! A production-ready Rust SDK for the Odos protocol - a decentralized exchange aggregator
//! that provides optimal routing for token swaps across multiple EVM chains.
//!
//! ## Features
//!
//! - **Multi-chain Support**: 16+ EVM chains including Ethereum, Arbitrum, Optimism, Polygon, Base, etc.
//! - **Type-safe**: Leverages Rust's type system with Alloy primitives for addresses, chain IDs, and amounts
//! - **Production-ready**: Built-in retry logic, circuit breakers, timeouts, and error handling
//! - **Builder Pattern**: Ergonomic API using the `bon` crate for request building
//! - **Comprehensive Error Handling**: Detailed error types for different failure scenarios
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use odos_sdk::*;
//! use alloy_primitives::{Address, U256};
//! use std::str::FromStr;
//!
//! # async fn example() -> Result<()> {
//! // Create a client
//! let client = OdosSorV2::new()?;
//!
//! // Build a quote request
//! let quote_request = QuoteRequest::builder()
//!     .chain_id(1) // Ethereum mainnet
//!     .input_tokens(vec![(
//!         Address::from_str("0xA0b86a33E6441d35a6b083d5b02a8e3F6CE21a2E")?, // WETH
//!         U256::from(1000000000000000000u64) // 1 ETH
//!     ).into()])
//!     .output_tokens(vec![(
//!         Address::from_str("0xA0b86a33E6441d35a6b083d5b02a8e3F6CE21a2E")?, // USDC
//!         1
//!     ).into()])
//!     .slippage_limit_percent(1.0)
//!     .user_addr("0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0".to_string())
//!     .compact(false)
//!     .simple(false)
//!     .referral_code(0)
//!     .disable_rfqs(false)
//!     .build();
//!
//! // Get a quote
//! let quote = client.get_swap_quote(&quote_request).await?;
//!
//! // Build transaction data
//! let swap_context = SwapContext::builder()
//!     .chain(alloy_chains::NamedChain::Mainnet)
//!     .router_address(alloy_chains::NamedChain::Mainnet.v2_router_address()?)
//!     .signer_address(Address::from_str("0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0")?)
//!     .output_recipient(Address::from_str("0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0")?)
//!     .token_address(Address::from_str("0xA0b86a33E6441d35a6b083d5b02a8e3F6CE21a2E")?)
//!     .token_amount(U256::from(1000000000000000000u64))
//!     .path_id(quote.path_id().to_string())
//!     .build();
//!
//! let transaction = client.build_base_transaction(&swap_context).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Configuration
//!
//! The SDK supports extensive configuration for production use:
//!
//! ```rust,no_run
//! use odos_sdk::*;
//! use std::time::Duration;
//!
//! # fn example() -> Result<()> {
//! let config = ClientConfig {
//!     timeout: Duration::from_secs(30),
//!     connect_timeout: Duration::from_secs(10),
//!     max_retries: 3,
//!     initial_retry_delay: Duration::from_millis(100),
//!     max_retry_delay: Duration::from_secs(5),
//!     max_connections: 20,
//!     pool_idle_timeout: Duration::from_secs(90),
//! };
//!
//! let client = OdosSorV2::with_config(config)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! The SDK provides comprehensive error types for different failure scenarios:
//!
//! ```rust,no_run
//! use odos_sdk::*;
//!
//! # async fn example() {
//! # let client = OdosSorV2::new().unwrap();
//! # let quote_request = QuoteRequest::builder().chain_id(1).input_tokens(vec![]).output_tokens(vec![]).slippage_limit_percent(1.0).user_addr("test".to_string()).compact(false).simple(false).referral_code(0).disable_rfqs(false).build();
//! match client.get_swap_quote(&quote_request).await {
//!     Ok(quote) => {
//!         // Handle successful quote
//!         println!("Got quote with path ID: {}", quote.path_id());
//!     }
//!     Err(OdosError::Api { status, message }) => {
//!         // Handle API errors
//!         eprintln!("API error {}: {}", status, message);
//!     }
//!     Err(OdosError::Timeout(msg)) => {
//!         // Handle timeout errors (retryable)
//!         eprintln!("Request timed out: {}", msg);
//!     }
//!     Err(OdosError::RateLimit(msg)) => {
//!         // Handle rate limiting (retryable)
//!         eprintln!("Rate limited: {}", msg);
//!     }
//!     Err(err) => {
//!         // Handle other errors
//!         eprintln!("Error: {}", err);
//!     }
//! }
//! # }
//! ```
//!
//! ## Rate Limiting
//!
//! The Odos API enforces rate limits to ensure fair usage. This SDK handles rate limiting automatically:
//!
//! - **HTTP 429 responses** are detected and classified as [`OdosError::RateLimit`]
//! - Requests are **automatically retried** with exponential backoff
//! - The SDK **respects `Retry-After` headers** when provided by the API
//! - Default configuration: **3 retry attempts** with exponential backoff (100ms, 200ms, 400ms)
//!
//! ### Best Practices for Avoiding Rate Limits
//!
//! 1. **Share a single client** across your application instead of creating new clients per request
//! 2. **Implement application-level rate limiting** if making many concurrent requests
//! 3. **Handle rate limit errors gracefully** and back off at the application level if needed
//!
//! ### Example: Handling Rate Limits
//!
//! ```rust,no_run
//! use odos_sdk::*;
//! use alloy_primitives::{Address, U256};
//! use std::time::Duration;
//!
//! # async fn example() -> Result<()> {
//! # let client = OdosSorV2::new()?;
//! # let quote_request = QuoteRequest::builder()
//! #     .chain_id(1)
//! #     .input_tokens(vec![])
//! #     .output_tokens(vec![])
//! #     .slippage_limit_percent(1.0)
//! #     .user_addr("test".to_string())
//! #     .compact(false)
//! #     .simple(false)
//! #     .referral_code(0)
//! #     .disable_rfqs(false)
//! #     .build();
//! match client.get_swap_quote(&quote_request).await {
//!     Ok(quote) => {
//!         println!("Got quote: {}", quote.path_id());
//!     }
//!     Err(e) if e.is_rate_limit() => {
//!         // Rate limit exceeded even after SDK retries
//!         // Consider backing off at application level
//!         eprintln!("Rate limited - waiting before retry");
//!         tokio::time::sleep(Duration::from_secs(5)).await;
//!         // Retry or handle accordingly
//!     }
//!     Err(e) => {
//!         eprintln!("Error: {}", e);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Configuring Retry Behavior
//!
//! You can customize retry behavior for your use case:
//!
//! ```rust,no_run
//! use odos_sdk::*;
//! use std::time::Duration;
//!
//! # fn example() -> Result<()> {
//! let config = ClientConfig {
//!     max_retries: 5,  // Increase from default 3
//!     initial_retry_delay: Duration::from_millis(200),
//!     max_retry_delay: Duration::from_secs(10),
//!     ..Default::default()
//! };
//!
//! let client = OdosSorV2::with_config(config)?;
//! # Ok(())
//! # }
//! ```
//!
//! **Trade-offs:**
//! - **Higher retry counts**: More resilient to transient rate limits, but slower failure in persistent scenarios
//! - **Longer delays**: Less likely to hit rate limits again, but slower overall throughput

mod api;
mod assemble;
mod chain;
mod client;
mod contract;
mod error;
#[cfg(test)]
mod integration_tests;
mod limit_order_v2;
mod sor;
mod swap;
mod transfer;
mod v2_router;
mod v3_router;

pub use api::*;
pub use assemble::*;
pub use chain::*;
pub use client::*;
pub use contract::*;
pub use error::*;
pub use limit_order_v2::*;
pub use sor::*;
pub use swap::*;
pub use transfer::*;
pub use v2_router::*;
pub use v3_router::*;
