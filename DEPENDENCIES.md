# Dependency Documentation

This document explains the rationale behind all dependencies in the odos-sdk crate, their purpose, and version choices.

## Philosophy

The odos-sdk follows these dependency management principles:

1. **Minimal footprint**: Only include dependencies that are absolutely necessary
2. **Stable ecosystem**: Prefer well-maintained crates with strong community support
3. **Version alignment**: Keep related dependencies (e.g., alloy-*) at compatible versions
4. **Security first**: Regular audits and prompt updates for security advisories
5. **Zero unnecessary transitive deps**: Don't explicitly declare dependencies that are already pulled in transitively

## Core Dependencies

### Alloy Ecosystem (Ethereum Interactions)

The SDK heavily relies on the [Alloy](https://github.com/alloy-rs/alloy) suite of crates for type-safe Ethereum interactions.

#### alloy-chains (0.2.x)

- **Purpose**: Chain ID definitions and type-safe chain support
- **Why this version**: alloy-chains is in a separate repository (github.com/alloy-rs/chains) with its own versioning - 0.2.x is the latest stable series and is fully compatible with alloy 1.x
- **Current version**: 0.2.25 (latest)
- **Usage**: Provides `NamedChain` enum and chain-specific configurations
- **Note**: Despite different major version numbers, alloy-chains 0.2.x is designed to work with alloy 1.x

#### alloy-primitives (1.5.x)

- **Purpose**: Core Ethereum types (`Address`, `U256`, `Bytes`, etc.)
- **Features enabled**:
  - `std`: Standard library support
  - `rlp`: RLP encoding/decoding for Ethereum data structures
- **Why this version**: Latest stable release with excellent performance and ergonomics
- **Usage**: All address handling, amount calculations, and hex encoding

#### alloy-contract (1.3.x)

- **Purpose**: Contract interaction abstractions
- **Features**: `default-features = false` for fine-grained control
- **Usage**: Used for contract bindings and transaction assembly

#### alloy-sol-types (1.5.x)

- **Purpose**: Solidity type definitions and ABI encoding/decoding
- **Features enabled**: `json` for ABI JSON support
- **Usage**: V2/V3 router contract ABIs and transaction encoding

#### alloy-sol-type-parser (1.5.x)

- **Purpose**: Parsing Solidity type strings at compile time
- **Usage**: Contract macro expansion for type-safe ABI bindings

#### alloy-provider (1.3.x)

- **Purpose**: RPC provider abstractions for Ethereum node communication
- **Features enabled**:
  - `anvil-node`: Local testing support
  - `reqwest`: HTTP transport via reqwest
- **Usage**: Transaction simulation and submission

#### alloy-signer (1.3.x), alloy-signer-local (1.3.x)

- **Purpose**: Transaction signing abstractions and local wallet support
- **Features**: `default-features = false` to avoid unnecessary dependencies
- **Usage**: Transaction signing for swap execution

#### alloy-network (1.3.x)

- **Purpose**: Network-level abstractions (mainnet, testnet, etc.)
- **Features**: `default-features = false`
- **Usage**: Network-aware transaction building

#### alloy-rpc-* crates (1.3.x)

- **alloy-rpc-client**: Low-level RPC client
- **alloy-rpc-types**: RPC type definitions
- **alloy-json-rpc**: JSON-RPC protocol support
- **Purpose**: HTTP communication with Ethereum nodes and services
- **Usage**: Contract calls and transaction submission

#### alloy-transport (1.3.x), alloy-transport-http (1.3.x)

- **Purpose**: Transport layer for RPC communications
- **Features**: `reqwest` for HTTP support
- **Usage**: HTTP communication layer

**Version Strategy**: The alloy ecosystem consists of two repositories:

1. **alloy-rs/alloy** (1.x): Main alloy crates for Ethereum interactions
2. **alloy-rs/chains** (0.2.x): Chain definitions maintained separately

Both are fully compatible despite different version numbers. We use the latest stable from each repository.

### HTTP Client

#### reqwest (0.12.x)

- **Purpose**: Async HTTP client for Odos API communication
- **Features enabled**: `json` for automatic JSON serialization
- **Why this crate**: Industry standard, excellent async support, built on hyper
- **Usage**: All HTTP requests to Odos API endpoints
- **Note**: Reqwest pulls in `tower` and `tower-http` as transitive dependencies for middleware support

### Retry Logic

#### backon (1.6.x)

- **Purpose**: Exponential backoff algorithm for retries
- **Why this crate**: Actively maintained alternative to the unmaintained `backoff` crate
- **Migration**: Migrated from `backoff` 0.4.x (unmaintained, RUSTSEC-2025-0012)
- **Features**: Iterator-based backoff API with configurable min/max delays
- **Usage**: Retry logic in `OdosHttpClient::execute_with_retry` for transient failures
- **API differences**: Uses `BackoffBuilder` trait and iterator pattern instead of mutable state

### Serialization

#### serde (1.0.x)

- **Purpose**: Serialization/deserialization framework
- **Features enabled**: `derive` for automatic derive macros
- **Why this crate**: De facto standard for Rust serialization
- **Usage**: All API request/response structures

#### serde_json (1.0.x)

- **Purpose**: JSON implementation for serde
- **Why this crate**: Fast, reliable, standard JSON support
- **Usage**: JSON encoding/decoding for Odos API

### Error Handling

#### thiserror (2.0.x)

- **Purpose**: Ergonomic error type derivation
- **Why this crate**: Best-in-class error handling macros
- **Usage**: `OdosError` enum and all error types throughout the SDK

### Async Runtime

#### tokio (1.0.x)

- **Purpose**: Async runtime for async/await support
- **Features enabled**:
  - `time`: Timer support for timeout and retry logic
  - `rt`: Runtime support
- **Why this crate**: Industry standard async runtime
- **Usage**: Async operations, timeouts, sleep in retry logic
- **Note**: Minimal feature set - we don't need `macros`, `fs`, or `net`

### Ergonomics

#### bon (3.x)

- **Purpose**: Builder pattern macro for ergonomic APIs
- **Why this crate**: Type-safe builders with excellent compile-time checking
- **Usage**: `QuoteRequest`, `SwapContext`, and other builder APIs
- **Alternative considered**: `derive_builder`, but `bon` has better ergonomics

### Observability

#### tracing (0.1.x)

- **Purpose**: Structured logging and instrumentation
- **Why this crate**: Standard for Rust observability
- **Usage**: Instrumentation throughout the SDK for debugging and monitoring
- **Future**: Will be made optional via feature flag in 1.0.0

### Utilities

#### url (2.5.x)

- **Purpose**: URL parsing and manipulation
- **Why this crate**: Standard URL handling
- **Usage**: API endpoint construction and validation

#### uuid (1.x)

- **Purpose**: UUID generation and handling
- **Features enabled**:
  - `serde`: Serialization support
  - `v4`: Random UUID generation
- **Usage**: Request tracking and correlation IDs

## Dev Dependencies

### wiremock (0.6.x)

- **Purpose**: HTTP mocking for tests
- **Why this crate**: Best mock server library for Rust
- **Usage**: All integration tests mock Odos API responses

### tokio-test (0.4.x)

- **Purpose**: Testing utilities for async code
- **Why this crate**: Official tokio test utilities
- **Usage**: Async test helpers and assertions

### http (1.3.x)

- **Purpose**: HTTP types and abstractions
- **Why this crate**: Standard HTTP types used by test utilities
- **Usage**: Test request/response construction

## Removed Dependencies

### Tower/Tower-HTTP (Previously Explicit, Now Transitive Only)

- **Removed in**: 1.0.0
- **Rationale**: These were explicitly declared but never used directly in our code
  - `tower` is pulled in by `reqwest`, `alloy-rpc-client`, and `alloy-transport`
  - `tower-http` is pulled in by `reqwest`
  - We don't use any tower middleware or services directly
- **Impact**: None - still available transitively

## Transitive Dependencies of Note

These are NOT explicitly declared but come from our direct dependencies:

### derivative (via ruint/ark-ff) - **[UNMAINTAINED]**

- **Advisory**: RUSTSEC-2024-0388
- **Status**: Pulled in by alloy's `ruint` dependency
- **Action**: No action needed - this is a proc macro crate with minimal runtime impact
- **Tracking**: Monitor alloy ecosystem for migration to maintained alternative

### instant **[REMOVED]**

- **Previously**: Transitive dependency via unmaintained `backoff` crate
- **Status**: Removed as part of migration from `backoff` to `backon`
- **Advisory**: RUSTSEC-2024-0384 (no longer applicable)

### paste (via syn-solidity) - **[UNMAINTAINED]**

- **Advisory**: RUSTSEC-2024-0436
- **Status**: Pulled in by alloy's Solidity parsing crates
- **Action**: No action needed - this is a proc macro crate
- **Note**: Despite "unmaintained" status, `paste` is feature-complete and widely used

## Dependency Update Policy

### Regular Updates

- **Frequency**: Weekly dependency checks via Dependabot
- **Security**: Immediate updates for security advisories
- **Major versions**: Careful evaluation with changelog review

### Breaking Changes

- **Pre-1.0**: Breaking dependency updates are acceptable
- **Post-1.0**: Breaking dependency updates require major version bump
- **Notification**: CHANGELOG will document all significant dependency changes

### CI Enforcement

- `cargo audit` runs on every PR - fails on vulnerabilities
- `cargo deny` checks licenses and advisories
- `cargo outdated` runs weekly to check for updates

## Future Improvements (Post-1.0 Roadmap)

1. **Feature Flags** (1.0.0):
   - Make `tracing` optional
   - Make `alloy-provider` optional for users who only want quote fetching
   - Add `full` feature for all functionality

2. **Dependency Reduction** (Completed in 1.0.0):
   - ✅ Replaced `backoff` with `backon`
   - ✅ Removed `anyhow` compatibility shim
   - ✅ Removed unused `tower`/`tower-http` explicit dependencies
   - Evaluate if `uuid` is strictly necessary or if we can use simpler correlation IDs

3. **Performance**:
   - Consider `simd-json` for faster JSON parsing (feature-gated)
   - Evaluate `rustls` vs `native-tls` for better performance

4. **Observability**:
   - Optional `metrics` crate integration
   - Optional `opentelemetry` integration

## License Compatibility

All dependencies are compatible with the project's Apache-2.0 license:

- Most are dual-licensed MIT/Apache-2.0
- None have copyleft licenses (GPL, LGPL, etc.)
- CI enforces license compatibility via `cargo-deny`

## Acknowledgments

This project builds on the excellent work of the Rust and Ethereum communities, particularly:

- The [Alloy](https://github.com/alloy-rs/alloy) project for type-safe Ethereum interactions
- The Tokio project for async runtime
- The Serde project for serialization

---

**Last Updated**: 2025-01-08 (Version 1.2.0)
**Next Review**: Before 1.3.0 release
