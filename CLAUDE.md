# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`odos-sdk` is a published Rust SDK (crates.io) for the Odos decentralized exchange aggregator, covering quote, transaction assembly, and on-chain execution helpers across 13+ EVM chains. The crate is built on the Alloy ecosystem (`~1.7`) and targets Rust 1.92.

## Essential Commands

```bash
cargo build
cargo test                                                   # unit + doc tests
cargo test --lib --no-default-features --features=<feature>  # match CI matrix
cargo test --test fork_tests                                 # anvil-backed fork tests
cargo test test_name                                         # single test
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings     # CI lint (zero warnings)
cargo doc --no-deps
cargo audit
```

CI runs the test matrix against feature sets `default`, `minimal`, `v2`, `v3`, `limit-orders`, `contracts` — when touching feature-gated code, verify the relevant combinations locally.

## Architecture

### Feature-flag layering (`Cargo.toml`)

The crate is sliced by feature flag and **modules in `lib.rs` are `#[cfg]`-gated to match**. Keep re-exports and doctests consistent with this layering:

- `minimal` — core API types + HTTP client + `tooling` DTOs only (no contract bindings, no on-chain helpers). Used by AI/tool integrations.
- `v2` — base contract feature; provides `SwapInputs` used by all routers.
- `v3` — V3 bindings (requires `v2`).
- `limit-orders` — limit-order bindings (requires `v2`).
- `multicall` — on-chain balance/allowance and preflight helpers.
- `contracts` — convenience: all contract features + `multicall`.
- `default` — `v2 + v3 + multicall`.

### Three-tier API

1. **High-level**: `SwapBuilder` (via `OdosClient::swap()`) — chain, tokens, slippage, signer, one `.build_transaction()` call.
2. **Mid-level**: `OdosClient::quote()` → `OdosClient::assemble()` using `QuoteRequest` / `AssemblyRequest` builders.
3. **Low-level**: Direct router contract bindings (`V2Router`, `V3Router`, `LimitOrderV2`).

`OdosClient` is the preferred entry point; `OdosSor` remains as a deprecated alias. The `prelude` module re-exports the high-level surface.

### Key modules (`src/`)

- `client.rs` — `OdosHttpClient`, `ClientConfig`, `RetryConfig` (connection pooling, timeouts, exponential backoff; rate-limit 429s are **never** retried).
- `sor.rs` — `OdosClient` / `OdosSor` smart-order-router client.
- `swap_builder.rs` — high-level `SwapBuilder`.
- `swap.rs` / `assemble.rs` — `SwapContext`, `AssemblyRequest`, assembled `TransactionData`.
- `api.rs` — wire DTOs: `QuoteRequest`, `SingleQuoteResponse`, `SwapInputs`, `Endpoint`, `ApiHost`, `ApiVersion`.
- `chain.rs` — `OdosChain` trait + `RouterAvailability` (per-chain v2/v3/lo router lookup).
- `contract.rs` — immutable router addresses and `get_supported_*_chains()` helpers.
- `types/` — domain types: `Chain`, `Slippage`, `ReferralCode`.
- `error.rs` / `error_code.rs` — `OdosError` (with trace-ID + `Retry-After`) and strongly-typed `OdosErrorCode` categories (1XXX general, 2XXX algo/quote, 3XXX internal service, 4XXX validation, 5XXX internal).
- `tooling.rs` — stable JSON DTOs for AI-agent/runtime integrations (available under `minimal`).
- `events.rs` — swap event filters/decoders (`v2` or `v3` feature).
- `multicall.rs`, `transfer.rs`, `router_type.rs`, `api_key.rs`, `limit_order_v2.rs` — feature-gated helpers.
- `v2_router.rs` / `v3_router.rs` — `sol!`-generated bindings; ABIs live in `abis/` and are `include_str!`ed.

### Design conventions

- **Builder pattern** via the `bon` crate — extend existing builders rather than introducing ad-hoc constructors.
- **Type safety** through Alloy primitives (`Address`, `U256`, `NamedChain`) — avoid string addresses or raw `u64` chain IDs in public APIs.
- **Errors** use `thiserror`; create the most specific variant at the source. Don't ignore `Result` with `_`.
- **Instrumentation** uses `tracing` with named fields (e.g. `tracing::debug!(chain_id, amount, ...)`), not string interpolation or `[tag]` prefixes.

## Repository conventions

- **Conventional Commits** are CI-enforced (commitizen); breaking changes use `!` or `BREAKING CHANGE:`.
- **REUSE compliance** is CI-enforced — every source file needs `SPDX-FileCopyrightText` + `SPDX-License-Identifier: Apache-2.0` headers (see existing files for the pattern).
- **CHANGELOG.md** must be updated under `[Unreleased]` for user-visible changes.
- **Releases** are tag-driven: bumping `Cargo.toml` + tagging `vX.Y.Z` triggers the crates.io publish workflow.
- **Contract ABIs** in `abis/` are compiled into bindings at build time — regenerate bindings when ABIs change.
- Tests are primarily in-module (`#[cfg(test)] mod tests`); integration tests live in `src/integration_tests.rs` and `tests/fork_tests.rs` (anvil-backed).
