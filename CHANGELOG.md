# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0] - 2026-01-12

### Removed

- **BREAKING**: Removed Mode chain support (chain ID 34443)
  - Odos has discontinued support for Mode
  - Removed `Chain::mode()` constructor
  - Removed `ODOS_V2_MODE_ROUTER` and `ODOS_LO_MODE_ROUTER` constants
  - Removed Mode from OP-stack chain detection
  - Users referencing Mode must migrate to other supported chains

## [1.3.0] - 2026-01-08

### Added

- **Error Classification Helper**: New `is_unroutable_token` method on `OdosErrorCode`
  - Identifies token routing errors for metrics classification
  - Useful for distinguishing between temporary failures and token-specific issues

### Changed

- **Dependencies**: Updated Alloy ecosystem packages
  - alloy-* packages: 1.2.1 → 1.3.0
  - alloy-chains: 0.2.24 → 0.2.25
  - Other minor dependency updates

## [1.2.0] - 2025-01-06

### Added

- **Router Type Detection**: Helper methods to distinguish swap vs order routers
  - `RouterType::is_swap_router()`: Check if router handles swaps (V2/V3)
  - `RouterType::is_order_router()`: Check if router handles limit orders
  - Useful for routing logic and contract interaction decisions

## [1.1.0] - 2025-01-04

### Added

- **OP-Stack Chain Support**: New `op-stack` feature flag for L2 chains
  - Base, Optimism, Fraxtal support with op-alloy v0.23.1
  - Access to L1 gas information in transaction receipts
  - New `op_stack` module with `Optimism` network type

- **Network-Generic Routers**: V2Router and V3Router now generic over Network trait
  - Default to `Ethereum` for backward compatibility
  - Support for any Alloy-compatible network type

- **Event Monitoring Utilities**: New `events` module
  - `SwapEventFilter` builder for querying swap events
  - Typed `SwapEvent` and `SwapMultiEvent` decoding
  - Support for both V2 and V3 router events

- **Multicall Utilities**: New `multicall` module with dual approach
  - Simple parallel RPC functions (`check_balance`, `check_allowance`) for 1-5 calls
  - Multicall3 batching (`multicall_check_balances`, etc.) for 10+ calls
  - `SwapPreflightCheck` and `PreflightResult` types

- **Fork Testing Infrastructure**: New Anvil-based integration tests
  - Tests against forked mainnet state
  - Router verification tests

- **Provider Documentation**: Comprehensive examples in lib.rs
  - Provider construction patterns
  - AnyNetwork usage
  - WebSocket subscription examples

### Changed

- **Dependencies**: Upgraded Alloy ecosystem
  - alloy-* packages: 1.1.0 → 1.2.1
  - alloy-primitives: 1.4.1 → 1.5.2
  - alloy-chains: 0.2.17 → 0.2.24

### Fixed

- **Security**: Update ruint to 1.17.2 to resolve RUSTSEC-2025-0137

## [1.0.0-beta.2] - 2025-11-10

### Removed

- **BREAKING**: Removed Berachain and Fantom chain support
  - Odos has discontinued support for these chains
  - Removed all chain configuration, router addresses, and tests
  - Updated documentation to reflect 15 supported chains
  - Users referencing these chains must migrate to other supported chains

## [1.0.0-beta.1] - 2025-11-10

### Added

- **MIGRATION.md**: Comprehensive migration guide from 0.x to 1.0
  - Step-by-step migration instructions
  - Before/after code examples
  - Feature flag migration guidance
  - SwapBuilder adoption guide

### Documentation

- **README.md**: Complete rewrite with user-focused approach
  - Value-first presentation showcasing optimal routing and production features
  - Three-tiered API explanation (SwapBuilder, quote+assemble, contracts)
  - Comprehensive feature showcase with code examples
  - Clear navigation to other documentation resources
  - Multi-chain support table and advanced topics

- **GETTING_STARTED.md**: New comprehensive tutorial (524 lines)
  - Installation and feature flag guide
  - Core concepts explanation (Client, Chains, Tokens, Slippage)
  - Complete walkthrough from setup to first swap
  - Understanding quotes and transaction execution
  - Production-ready error handling patterns
  - Common issues and troubleshooting
  - Token approval workflow

- **EXAMPLES.md**: New production patterns guide (900+ lines)
  - Basic swaps (simple, custom recipient, dynamic slippage)
  - Advanced quote handling (quote-first, multi-chain comparison, timeouts)
  - Comprehensive error handling patterns
  - Multi-chain operations and router selection
  - Testing strategies with mock patterns
  - Production patterns (singleton client, connection pooling, graceful shutdown)
  - Integration examples (Axum web server, CLI tool)
  - Best practices summary

All documentation is copy-paste ready with progressive complexity from quick wins to advanced patterns.

## [0.27.0] - 2025-11-10

### Changed

- **Type Safety**: Updated to use `Address` type for token addresses and user addresses
  - `QuoteRequest::user_addr` now accepts `Address` directly instead of `String`
  - Improved type safety and consistency with Alloy primitives
  - Migration: Parse strings to `Address` before passing: `"0x...".parse::<Address>()?`

### Fixed

- Improved type consistency across API surface

## [0.26.0] - 2025-11-08

### Added

- **Error Context Helpers**: New convenience methods on `OdosError`
  - `suggested_retry_delay()`: Returns recommended backoff duration for retryable errors
  - `is_client_error()`: Checks if error is a 4xx API error
  - `is_server_error()`: Checks if error is a 5xx API error

- **Prelude Module**: Convenient single-import for common types
  - `use odos_sdk::prelude::*;` imports:
    - Core clients: `OdosClient`, `SwapBuilder`
    - Request/Response types: `QuoteRequest`, `AssemblyRequest`, `SingleQuoteResponse`
    - Domain types: `Chain`, `Slippage`, `ReferralCode`
    - Error types: `OdosError`, `Result`
    - Alloy primitives: `Address`, `U256`

### Documentation

- Updated `lib.rs` documentation with SwapBuilder examples
- Added prelude usage examples throughout
- Improved quick start with both high-level and low-level patterns

## [0.25.0] - 2025-11-08

### Added

- **Granular Feature Flags**: Fine-grained control over contract bindings
  - `minimal`: Core API types and HTTP client only (no contract bindings)
  - `v2`: V2 router contract bindings
  - `v3`: V3 router contract bindings (includes v2 for SwapInputs type)
  - `limit-orders`: Limit order contract bindings (includes v2)
  - `contracts`: Convenience feature for all contract bindings
  - `default`: V2 + V3 routers (most common use case)

### Changed

- **Conditional Exports**: API surface now conditional on feature flags
  - `SwapInputs` only available with `v2` feature
  - Router contract types conditional on respective features
  - Reduced compile times and dependencies when using minimal feature

### Documentation

- Added feature flag documentation to README
- Examples for different feature combinations
- CI now tests all feature combinations

## [0.24.2] - 2025-11-08

### Added

- **High-Level SwapBuilder API**: Ergonomic builder for common swap operations
  - Fluent interface with method chaining
  - Simplifies quote → assemble → build flow from 15+ lines to 5 lines
  - Methods: `chain()`, `from_token()`, `to_token()`, `slippage()`, `signer()`, `recipient()`, `referral()`, `compact()`, `simple()`
  - `build_transaction()`: Returns TransactionRequest ready for execution
  - `build_context()`: Returns SwapContext for low-level access
  - `quote()`: Returns quote without building transaction
  - Integrates with type-safe domain types (Chain, Slippage, ReferralCode)

- **OdosClient::swap()**: Convenience method returning SwapBuilder

### Documentation

- Added SwapBuilder examples to documentation
- Comparison of high-level vs low-level API patterns

## [0.24.1] - 2025-11-08

### Added

- Exposed `change_liquidator_address` method on `LimitOrderV2` contract wrapper

### Fixed

- Missing contract method prevented consumers from calling `changeLiquidatorAddress` on the Limit Order V2 router

## [1.0.0] - 2025-01-02

### Removed (Breaking Changes)

- **BREAKING**: Removed deprecated `EndpointBase` enum (deprecated since 0.21.0)
  - **Migration**: Use `ApiHost` enum instead
- **BREAKING**: Removed deprecated `EndpointVersion` enum (deprecated since 0.21.0)
  - **Migration**: Use `ApiVersion` enum instead
- **BREAKING**: Use `Endpoint` convenience constructors instead of separate fields
  - **Migration**: `Endpoint::public_v2()`, `Endpoint::enterprise_v3()`, etc.
- Removed `anyhow` backwards compatibility shim

### Changed

- **Dependency Migration**: Replaced unmaintained `backoff` (0.4) with `backon` (1.6)
  - **Security**: Fixes RUSTSEC-2025-0012 (backoff unmaintained)
  - **Security**: Fixes RUSTSEC-2024-0384 (instant unmaintained)
- Updated alloy ecosystem dependencies to latest stable versions

### Added

- **DEPENDENCIES.md**: Comprehensive documentation of all dependencies

## [0.12.0] - 2025-10-27

### Added

- **RetryConfig**: New configuration struct for controlling retry behavior
  - `max_retries`: Maximum number of retry attempts
  - `initial_backoff_ms`: Initial backoff duration in milliseconds
  - `retry_server_errors`: Whether to retry 5xx server errors
  - `retry_predicate`: Optional custom retry logic function
- **RetryConfig presets**:
  - `RetryConfig::default()`: Default retry behavior (3 retries, retry server errors)
  - `RetryConfig::no_retries()`: Disable all retries
  - `RetryConfig::conservative()`: Only retry network errors (2 retries, no server error retries)
- **ClientConfig presets**:
  - `ClientConfig::no_retries()`: Client with no retry logic
  - `ClientConfig::conservative()`: Client with conservative retry behavior
- **OdosSor::with_retry_config()**: Convenience constructor for custom retry configuration
- **OdosError::retry_after()**: Helper method to extract Retry-After duration from rate limit errors
- **Smart retry logic**: The SDK now intelligently determines which errors should be retried:
  - ✅ Network errors (timeouts, connection failures) are retried
  - ✅ Server errors (5xx) are conditionally retried based on configuration
  - ❌ Rate limit errors (429) are NOT retried
  - ❌ Client errors (4xx) are NOT retried

### Changed

- **BREAKING**: `OdosError::RateLimit` variant changed from tuple struct to named struct:
  - Old: `RateLimit(String)`
  - New: `RateLimit { message: String, retry_after: Option<Duration> }`
  - The `retry_after` field contains the value from the `Retry-After` HTTP header
- **BREAKING**: `ClientConfig` structure changed:
  - Removed: `max_retries`, `initial_retry_delay`, `max_retry_delay` fields
  - Added: `retry_config: RetryConfig` field
  - Migration: Replace direct field access with `config.retry_config.max_retries` etc.
- **BREAKING**: Rate limit errors (HTTP 429) are NO LONGER automatically retried
  - **Rationale**: Retrying rate limits creates a cascade effect that worsens the problem
  - **Migration**: Applications must handle rate limits globally with proper coordination
  - The SDK now returns rate limit errors immediately with the `Retry-After` header preserved
- Rate limit errors are no longer marked as retryable (`is_retryable()` returns `false`)

### Fixed

- Retry cascade problem when multiple concurrent requests hit rate limits
- Applications can now implement proper global rate limiting instead of per-request retries

### Migration Guide

#### 1. Update RateLimit Error Handling

**Before:**

```rust
match error {
    OdosError::RateLimit(msg) => {
        eprintln!("Rate limited: {}", msg);
    }
    ...
}
```

**After:**

```rust
match error {
    OdosError::RateLimit { message, retry_after } => {
        if let Some(duration) = retry_after {
            eprintln!("Rate limited: {}. Retry after {} seconds",
                message, duration.as_secs());
        } else {
            eprintln!("Rate limited: {}", message);
        }
    }
    ...
}
```

#### 2. Update ClientConfig Usage

**Before:**

```rust
let config = ClientConfig {
    max_retries: 5,
    initial_retry_delay: Duration::from_millis(200),
    max_retry_delay: Duration::from_secs(10),
    ..Default::default()
};
```

**After:**

```rust
let config = ClientConfig {
    retry_config: RetryConfig {
        max_retries: 5,
        initial_backoff_ms: 200,
        ..Default::default()
    },
    ..Default::default()
};

// Or use convenience constructors:
let config = ClientConfig::conservative();
```

#### 3. Handle Rate Limits Globally

**Before:** Rate limits were retried automatically (causing cascade issues)

**After:** Implement application-level rate limiting

```rust
// Option 1: Reduce concurrency
const MAX_CONCURRENT_REQUESTS: usize = 2;

// Option 2: Implement backoff when rate limited
match client.get_swap_quote(&request).await {
    Ok(quote) => { /* success */ }
    Err(e) if e.is_rate_limit() => {
        // Back off and retry at application level
        if let Some(duration) = e.retry_after() {
            tokio::time::sleep(duration).await;
        }
        // Retry with global coordination
    }
    Err(e) => { /* other errors */ }
}
```

## [0.11.0] - 2025-10-27

Previous releases...
