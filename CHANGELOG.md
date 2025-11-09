# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.24.1] - 2025-11-08

### Added

- Exposed `change_liquidator_address` method on `LimitOrderV2` contract wrapper

### Fixed

- Missing contract method prevented consumers from calling `changeLiquidatorAddress` on the Limit Order V2 router

## [1.0.0] - In Progress

### PHASE 1: FOUNDATION & CLEANUP ✅ COMPLETED

#### Removed (Breaking Changes)

- **BREAKING**: Removed deprecated `EndpointBase` enum (deprecated since 0.21.0)
  - **Migration**: Use `ApiHost` enum instead
  - Example: `EndpointBase::Public` → `ApiHost::Public`
- **BREAKING**: Removed deprecated `EndpointVersion` enum (deprecated since 0.21.0)
  - **Migration**: Use `ApiVersion` enum instead
  - Example: `EndpointVersion::V2` → `ApiVersion::V2`
- **BREAKING**: Use `Endpoint` convenience constructors instead of separate fields
  - **Migration**: `Endpoint::public_v2()`, `Endpoint::enterprise_v3()`, etc.
- Removed `From<EndpointBase>` for `ApiHost` trait implementation
- Removed `From<EndpointVersion>` for `ApiVersion` trait implementation
- Removed `anyhow` backwards compatibility shim (`From<anyhow::Error>` for `OdosError`)

#### Changed

- **Dependency Migration**: Replaced unmaintained `backoff` (0.4) with `backon` (1.6)
  - **Security**: Fixes RUSTSEC-2025-0012 (backoff unmaintained)
  - **Security**: Fixes RUSTSEC-2024-0384 (instant unmaintained, transitive via backoff)
  - **Impact**: Internal implementation detail, no API changes
  - Uses iterator-based backoff API with `BackoffBuilder` trait
- Updated `alloy-chains` from 0.2.16 to 0.2.17 (latest stable)
- Removed explicit `tower` and `tower-http` dependencies (available transitively via reqwest)

#### Added

- **DEPENDENCIES.md**: Comprehensive documentation of all 25+ dependencies
  - Rationale and purpose for each dependency
  - Version choices explained
  - Security considerations and audit results
  - Migration notes from unmaintained crates
  - Future improvement roadmap
  - License compatibility information

### PHASE 1: REMAINING TASKS (In Progress)

- [ ] Replace glob re-exports with explicit exports in lib.rs
- [ ] Fix broken rustdoc links in client.rs
- [ ] Add CONTRIBUTING.md with development workflow
- [ ] Add SECURITY.md with vulnerability reporting
- [ ] Fix README example to be compilable
- [ ] Run final clippy check with all lints

### PLANNED: PHASE 2-5 (8-12 weeks timeline)

See comprehensive 1.0.0 release plan for:
- Phase 2: API Redesign for Excellence (Weeks 3-5)
- Phase 3: Production Hardening (Weeks 6-8)
- Phase 4: Documentation & Developer Experience (Weeks 9-10)
- Phase 5: Testing & Validation (Weeks 11-12)

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
