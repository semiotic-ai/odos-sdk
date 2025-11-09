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
//! let client = OdosSor::new()?;
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
//! // Full configuration
//! let config = ClientConfig {
//!     timeout: Duration::from_secs(30),
//!     connect_timeout: Duration::from_secs(10),
//!     retry_config: RetryConfig {
//!         max_retries: 3,
//!         initial_backoff_ms: 100,
//!         retry_server_errors: true,
//!         retry_predicate: None,
//!     },
//!     max_connections: 20,
//!     pool_idle_timeout: Duration::from_secs(90),
//!     api_key: None,
//!     ..Default::default()
//! };
//! let client = OdosSor::with_config(config)?;
//!
//! // Or use convenience constructors
//! let client = OdosSor::with_retry_config(RetryConfig::conservative())?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! The SDK provides comprehensive error types with strongly-typed error codes:
//!
//! ```rust,no_run
//! use odos_sdk::*;
//!
//! # async fn example() {
//! # let client = OdosSor::new().unwrap();
//! # let quote_request = QuoteRequest::builder().chain_id(1).input_tokens(vec![]).output_tokens(vec![]).slippage_limit_percent(1.0).user_addr("test".to_string()).compact(false).simple(false).referral_code(0).disable_rfqs(false).build();
//! match client.get_swap_quote(&quote_request).await {
//!     Ok(quote) => {
//!         // Handle successful quote
//!         println!("Got quote with path ID: {}", quote.path_id());
//!     }
//!     Err(err) => {
//!         // Check for specific error codes
//!         if let Some(code) = err.error_code() {
//!             if code.is_invalid_chain_id() {
//!                 eprintln!("Invalid chain ID - check configuration");
//!             } else if code.is_no_viable_path() {
//!                 eprintln!("No routing path found");
//!             } else if code.is_timeout() {
//!                 eprintln!("Service timeout: {}", code);
//!             }
//!         }
//!
//!         // Log trace ID for support
//!         if let Some(trace_id) = err.trace_id() {
//!             eprintln!("Trace ID: {}", trace_id);
//!         }
//!
//!         // Handle by error type
//!         match err {
//!             OdosError::Api { status, message, .. } => {
//!                 eprintln!("API error {}: {}", status, message);
//!             }
//!             OdosError::Timeout(msg) => {
//!                 eprintln!("Request timed out: {}", msg);
//!             }
//!             OdosError::RateLimit { message, retry_after, .. } => {
//!                 if let Some(duration) = retry_after {
//!                     eprintln!("Rate limited: {}. Retry after {} seconds", message, duration.as_secs());
//!                 } else {
//!                     eprintln!("Rate limited: {}", message);
//!                 }
//!             }
//!             _ => eprintln!("Error: {}", err),
//!         }
//!     }
//! }
//! # }
//! ```
//!
//! ### Strongly-Typed Error Codes
//!
//! The SDK provides error codes matching the [Odos API documentation](https://docs.odos.xyz/build/api_errors):
//!
//! - **General (1XXX)**: `ApiError`
//! - **Algo/Quote (2XXX)**: `NoViablePath`, `AlgoTimeout`, `AlgoInternal`
//! - **Internal Service (3XXX)**: `TxnAssemblyTimeout`, `GasUnavailable`
//! - **Validation (4XXX)**: `InvalidChainId`, `BlockedUserAddr`, `InvalidTokenAmount`
//! - **Internal (5XXX)**: `InternalError`, `SwapUnavailable`
//!
//! ```rust,no_run
//! use odos_sdk::{OdosError, error_code::OdosErrorCode};
//!
//! # fn handle_error(error: OdosError) {
//! if let Some(code) = error.error_code() {
//!     // Check categories
//!     if code.is_validation_error() {
//!         println!("Validation error - check request parameters");
//!     }
//!
//!     // Check retryability
//!     if code.is_retryable() {
//!         println!("Error can be retried: {}", code);
//!     }
//! }
//! # }
//! ```
//!
//! ## Rate Limiting
//!
//! The Odos API enforces rate limits to ensure fair usage. The SDK handles rate limits intelligently:
//!
//! - **HTTP 429 responses** are detected and classified as [`OdosError::RateLimit`]
//! - Rate limit errors are **NOT retried** (return immediately with `Retry-After` header)
//! - The SDK **captures `Retry-After` headers** for application-level handling
//! - Applications should handle rate limits globally with proper backoff coordination
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
//! # let client = OdosSor::new()?;
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
//!
//! # fn example() -> Result<()> {
//! // Conservative: only retry network errors
//! let client = OdosSor::with_retry_config(RetryConfig::conservative())?;
//!
//! // No retries: handle all errors at application level
//! let client = OdosSor::with_retry_config(RetryConfig::no_retries())?;
//!
//! // Custom configuration
//! let retry_config = RetryConfig {
//!     max_retries: 5,
//!     initial_backoff_ms: 200,
//!     retry_server_errors: false,  // Don't retry 5xx errors
//!     retry_predicate: None,
//! };
//! let client = OdosSor::with_retry_config(retry_config)?;
//! # Ok(())
//! # }
//! ```
//!
//! **Note:** Rate limit errors (429) are never retried regardless of configuration.
//! This prevents retry cascades that make rate limiting worse.

mod api;
mod api_key;
mod assemble;
mod chain;
mod client;
mod contract;
mod error;
pub mod error_code;
#[cfg(test)]
mod integration_tests;
mod limit_order_v2;
mod router_type;
mod sor;
mod swap;
mod swap_builder;
mod transfer;
mod types;
mod v2_router;
mod v3_router;

// API types
pub use api::{
    ApiHost, ApiVersion, Endpoint, InputToken, OdosApiErrorResponse, OutputToken, QuoteRequest,
    SingleQuoteResponse, SwapInputs,
};

// API key management
pub use api_key::ApiKey;

// Transaction assembly
pub use assemble::{
    parse_value, AssembleRequest, AssemblyResponse, Simulation, SimulationError, TransactionData,
};

// Chain support
pub use chain::{OdosChain, OdosChainError, OdosChainResult, OdosRouterSelection};

// HTTP client configuration
pub use client::{ClientConfig, OdosHttpClient, RetryConfig};

// Contract addresses and chain helpers
pub use contract::{
    get_lo_router_by_chain_id, get_supported_chains, get_supported_lo_chains,
    get_supported_v2_chains, get_supported_v3_chains, get_v2_router_by_chain_id,
    get_v3_router_by_chain_id, ODOS_LO_ARBITRUM_ROUTER, ODOS_LO_AVALANCHE_ROUTER,
    ODOS_LO_BASE_ROUTER, ODOS_LO_BERACHAIN_ROUTER, ODOS_LO_BSC_ROUTER, ODOS_LO_ETHEREUM_ROUTER,
    ODOS_LO_FRAXTAL_ROUTER, ODOS_LO_LINEA_ROUTER, ODOS_LO_MANTLE_ROUTER, ODOS_LO_MODE_ROUTER,
    ODOS_LO_OP_ROUTER, ODOS_LO_POLYGON_ROUTER, ODOS_LO_SCROLL_ROUTER, ODOS_LO_SONIC_ROUTER,
    ODOS_LO_UNICHAIN_ROUTER, ODOS_LO_ZKSYNC_ROUTER, ODOS_V2_ARBITRUM_ROUTER,
    ODOS_V2_AVALANCHE_ROUTER, ODOS_V2_BASE_ROUTER, ODOS_V2_BSC_ROUTER, ODOS_V2_ETHEREUM_ROUTER,
    ODOS_V2_FRAXTAL_ROUTER, ODOS_V2_LINEA_ROUTER, ODOS_V2_MANTLE_ROUTER, ODOS_V2_MODE_ROUTER,
    ODOS_V2_OP_ROUTER, ODOS_V2_POLYGON_ROUTER, ODOS_V2_SCROLL_ROUTER, ODOS_V2_SONIC_ROUTER,
    ODOS_V2_UNICHAIN_ROUTER, ODOS_V2_ZKSYNC_ROUTER, ODOS_V3,
};

// Error handling
pub use error::{OdosError, Result};

// Limit order contract bindings
pub use limit_order_v2::LimitOrderV2;

// Router type selection
pub use router_type::{RouterAvailability, RouterType};

// Smart Order Router client
pub use sor::OdosSor;

// Swap execution context
pub use swap::SwapContext;

// High-level swap builder
pub use swap_builder::SwapBuilder;

// Transfer types
pub use transfer::TransferRouterFunds;

// Type-safe domain types
pub use types::{Chain, ReferralCode, Slippage};

// Contract bindings
pub use v2_router::{OdosRouterV2, OdosV2Router, V2Router};
pub use v3_router::{IOdosRouterV3, OdosV3Router, V3Router};
