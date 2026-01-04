// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
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
//! ### High-Level API with SwapBuilder
//!
//! The easiest way to get started is with the [`SwapBuilder`] API:
//!
//! ```rust,no_run
//! use odos_sdk::prelude::*;
//! use std::str::FromStr;
//!
//! # async fn example() -> Result<()> {
//! // Create a client
//! let client = OdosClient::new()?;
//!
//! // Define tokens and amount
//! let usdc = Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?; // USDC on Ethereum
//! let weth = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")?; // WETH on Ethereum
//! let my_address = Address::from_str("0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0")?;
//!
//! // Build and execute swap in one go
//! let transaction = client.swap()
//!     .chain(Chain::ethereum())
//!     .from_token(usdc, U256::from(1_000_000)) // 1 USDC (6 decimals)
//!     .to_token(weth)
//!     .slippage(Slippage::percent(0.5).unwrap()) // 0.5% slippage
//!     .signer(my_address)
//!     .build_transaction()
//!     .await?;
//!
//! println!("Transaction ready: {:?}", transaction);
//! # Ok(())
//! # }
//! ```
//!
//! ### Low-Level API
//!
//! For more control, use the low-level API with [`quote()`](OdosClient::quote) and [`assemble()`](OdosClient::assemble):
//!
//! ```rust,no_run
//! use odos_sdk::prelude::*;
//! use alloy_primitives::address;
//! use std::str::FromStr;
//!
//! # async fn example() -> Result<()> {
//! let client = OdosClient::new()?;
//! let usdc = Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?;
//! let weth = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")?;
//!
//! // Step 1: Get a quote
//! let quote_request = QuoteRequest::builder()
//!     .chain_id(1)
//!     .input_tokens(vec![(usdc, U256::from(1_000_000)).into()])
//!     .output_tokens(vec![(weth, 1).into()])
//!     .slippage_limit_percent(0.5)
//!     .user_addr(address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0"))
//!     .compact(false)
//!     .simple(false)
//!     .referral_code(0)
//!     .disable_rfqs(false)
//!     .build();
//!
//! let quote = client.quote(&quote_request).await?;
//! println!("Expected output: {} WETH", quote.out_amount().unwrap_or(&"0".to_string()));
//!
//! // Step 2: Assemble transaction
//! let assembly_request = AssemblyRequest::builder()
//!     .chain(alloy_chains::NamedChain::Mainnet)
//!     .router_address(alloy_chains::NamedChain::Mainnet.v2_router_address()?)
//!     .signer_address(Address::from_str("0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0")?)
//!     .output_recipient(Address::from_str("0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0")?)
//!     .token_address(usdc)
//!     .token_amount(U256::from(1_000_000))
//!     .path_id(quote.path_id().to_string())
//!     .build();
//!
//! let transaction = client.assemble(&assembly_request).await?;
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
//! let client = OdosClient::with_config(config)?;
//!
//! // Or use convenience constructors
//! let client = OdosClient::with_retry_config(RetryConfig::conservative())?;
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
//! use alloy_primitives::Address;
//!
//! # async fn example() {
//! # let client = OdosClient::new().unwrap();
//! # let quote_request = QuoteRequest::builder().chain_id(1).input_tokens(vec![]).output_tokens(vec![]).slippage_limit_percent(1.0).user_addr(Address::ZERO).compact(false).simple(false).referral_code(0).disable_rfqs(false).build();
//! match client.quote(&quote_request).await {
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
//! # let client = OdosClient::new()?;
//! # let quote_request = QuoteRequest::builder()
//! #     .chain_id(1)
//! #     .input_tokens(vec![])
//! #     .output_tokens(vec![])
//! #     .slippage_limit_percent(1.0)
//! #     .user_addr(Address::ZERO)
//! #     .compact(false)
//! #     .simple(false)
//! #     .referral_code(0)
//! #     .disable_rfqs(false)
//! #     .build();
//! match client.quote(&quote_request).await {
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
//! let client = OdosClient::with_retry_config(RetryConfig::conservative())?;
//!
//! // No retries: handle all errors at application level
//! let client = OdosClient::with_retry_config(RetryConfig::no_retries())?;
//!
//! // Custom configuration
//! let retry_config = RetryConfig {
//!     max_retries: 5,
//!     initial_backoff_ms: 200,
//!     retry_server_errors: false,  // Don't retry 5xx errors
//!     retry_predicate: None,
//! };
//! let client = OdosClient::with_retry_config(retry_config)?;
//! # Ok(())
//! # }
//! ```
//!
//! **Note:** Rate limit errors (429) are never retried regardless of configuration.
//! This prevents retry cascades that make rate limiting worse.
//!
//! ## Provider Construction (Alloy Best Practices)
//!
//! ### Dynamic Chain Support with AnyNetwork
//!
//! For applications that need to work with multiple chains dynamically (without knowing
//! the chain at compile time), use `AnyNetwork`:
//!
//! ```rust,ignore
//! use alloy_network::AnyNetwork;
//! use alloy_provider::ProviderBuilder;
//!
//! // Works with any EVM chain without network-specific types
//! let provider = ProviderBuilder::new()
//!     .network::<AnyNetwork>()
//!     .connect_http(rpc_url.parse()?);
//!
//! // Routers work with AnyNetwork
//! let router: V3Router<AnyNetwork, _> = V3Router::new(router_address, provider);
//! ```
//!
//! **Trade-offs:**
//! - ✅ Single code path for all chains
//! - ✅ Simpler multi-chain applications
//! - ⚠️ Loses network-specific receipt fields (e.g., OP-stack L1 gas info)
//! - ⚠️ Less compile-time type safety
//!
//! When executing swaps on-chain, use Alloy's [`ProviderBuilder`](alloy_provider::ProviderBuilder)
//! with recommended fillers for proper nonce management, gas estimation, and chain ID handling:
//!
//! ```rust,ignore
//! use alloy_provider::ProviderBuilder;
//! use alloy_signer_local::PrivateKeySigner;
//! use alloy_network::EthereumWallet;
//!
//! // Create a signer from a private key
//! let signer: PrivateKeySigner = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
//!     .parse()
//!     .expect("valid private key");
//!
//! // Create a provider with recommended fillers (nonce, gas, chain ID)
//! let provider = ProviderBuilder::new()
//!     .with_recommended_fillers()
//!     .wallet(EthereumWallet::new(signer))
//!     .connect_http("https://eth.llamarpc.com".parse()?);
//!
//! // Use the provider with routers
//! let router = odos_sdk::V3Router::new(router_address, provider);
//! ```
//!
//! ### OP-Stack Chains (Base, Optimism, Mode, Fraxtal)
//!
//! For OP-stack chains, use the `Optimism` network type to access L1 gas information:
//!
//! ```rust,ignore
//! #[cfg(feature = "op-stack")]
//! {
//!     use odos_sdk::op_stack::Optimism;
//!     use alloy_provider::ProviderBuilder;
//!
//!     // Create a provider for OP-stack chains
//!     let provider = ProviderBuilder::new()
//!         .with_recommended_fillers()
//!         .network::<Optimism>()
//!         .connect_http("https://mainnet.base.org".parse()?);
//!
//!     // Transaction receipts will include L1 gas information
//!     let receipt = provider.get_transaction_receipt(tx_hash).await?;
//!     if let Some(l1_fee) = receipt.inner.l1_fee {
//!         println!("L1 fee: {l1_fee}");
//!     }
//! }
//! ```
//!
//! ### Sharing Providers
//!
//! For concurrent swap operations, share a single provider instance:
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use alloy_provider::ProviderBuilder;
//!
//! // Create a shared provider
//! let provider = Arc::new(
//!     ProviderBuilder::new()
//!         .with_recommended_fillers()
//!         .connect_http("https://eth.llamarpc.com".parse()?)
//! );
//!
//! // Clone the Arc for each concurrent operation
//! let provider_clone = Arc::clone(&provider);
//! tokio::spawn(async move {
//!     // Use provider_clone for concurrent swap
//! });
//! ```
//!
//! ### WebSocket Providers for Real-Time Monitoring
//!
//! Use WebSocket providers for real-time swap event monitoring:
//!
//! ```rust,ignore
//! use alloy_provider::ProviderBuilder;
//! use alloy_rpc_types::Filter;
//! use odos_sdk::events::SwapEventFilter;
//! use futures_util::StreamExt;
//!
//! // Connect via WebSocket for subscriptions
//! let ws_provider = ProviderBuilder::new()
//!     .connect_ws("wss://eth.llamarpc.com".parse()?)
//!     .await?;
//!
//! // Create a filter for swap events
//! let filter = SwapEventFilter::new(router_address)
//!     .from_latest()
//!     .build_v3_filter();
//!
//! // Subscribe to real-time swap events
//! let mut stream = ws_provider.subscribe_logs(&filter).await?;
//!
//! while let Some(log) = stream.next().await {
//!     match log {
//!         Ok(log) => println!("New swap detected: {:?}", log),
//!         Err(e) => eprintln!("Subscription error: {e}"),
//!     }
//! }
//! ```

mod api;
mod api_key;
mod assemble;
mod chain;
mod client;
mod contract;
mod error;
pub mod error_code;
#[cfg(any(feature = "v2", feature = "v3"))]
pub mod events;
#[cfg(test)]
mod integration_tests;
#[cfg(feature = "limit-orders")]
mod limit_order_v2;
pub mod multicall;
#[cfg(feature = "op-stack")]
pub mod op_stack;
mod router_type;
mod sor;
mod swap;
mod swap_builder;
mod transfer;
mod types;

// Prelude for convenient imports
pub mod prelude;

#[cfg(feature = "v2")]
mod v2_router;
#[cfg(feature = "v3")]
mod v3_router;

// API types
pub use api::{
    ApiHost, ApiVersion, Endpoint, InputToken, OdosApiErrorResponse, OutputToken, QuoteRequest,
    SingleQuoteResponse,
};

// SwapInputs is only available with v2 feature (contains V2 router types)
#[cfg(feature = "v2")]
pub use api::SwapInputs;

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
    ODOS_LO_BASE_ROUTER, ODOS_LO_BSC_ROUTER, ODOS_LO_ETHEREUM_ROUTER, ODOS_LO_FRAXTAL_ROUTER,
    ODOS_LO_LINEA_ROUTER, ODOS_LO_MANTLE_ROUTER, ODOS_LO_MODE_ROUTER, ODOS_LO_OP_ROUTER,
    ODOS_LO_POLYGON_ROUTER, ODOS_LO_SCROLL_ROUTER, ODOS_LO_SONIC_ROUTER, ODOS_LO_UNICHAIN_ROUTER,
    ODOS_LO_ZKSYNC_ROUTER, ODOS_V2_ARBITRUM_ROUTER, ODOS_V2_AVALANCHE_ROUTER, ODOS_V2_BASE_ROUTER,
    ODOS_V2_BSC_ROUTER, ODOS_V2_ETHEREUM_ROUTER, ODOS_V2_FRAXTAL_ROUTER, ODOS_V2_LINEA_ROUTER,
    ODOS_V2_MANTLE_ROUTER, ODOS_V2_MODE_ROUTER, ODOS_V2_OP_ROUTER, ODOS_V2_POLYGON_ROUTER,
    ODOS_V2_SCROLL_ROUTER, ODOS_V2_SONIC_ROUTER, ODOS_V2_UNICHAIN_ROUTER, ODOS_V2_ZKSYNC_ROUTER,
    ODOS_V3,
};

// Error handling
pub use error::{OdosError, Result};

// Limit order contract bindings
#[cfg(feature = "limit-orders")]
pub use limit_order_v2::LimitOrderV2;

// Router type selection
pub use router_type::{RouterAvailability, RouterType};

// Smart Order Router client
#[allow(deprecated)]
pub use sor::{OdosClient, OdosSor};

// Swap execution context
#[allow(deprecated)]
pub use swap::{AssemblyRequest, SwapContext};

// High-level swap builder
pub use swap_builder::SwapBuilder;

// Transfer types
pub use transfer::TransferRouterFunds;

// Type-safe domain types
pub use types::{Chain, ReferralCode, Slippage};

// V2 router contract bindings
#[cfg(feature = "v2")]
pub use v2_router::{OdosRouterV2, OdosV2Router, V2Router};

// V3 router contract bindings
#[cfg(feature = "v3")]
pub use v3_router::{IOdosRouterV3, OdosV3Router, V3Router};
