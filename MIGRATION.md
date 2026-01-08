# Migration Guide: 0.x → 1.0

This guide helps you migrate from odos-sdk 0.x to 1.0. The 1.0 release includes significant API improvements focused on ergonomics, type safety, and clarity.

## Overview

**Breaking Changes:**

- Type renames for clarity
- Method renames for consistency
- New high-level API (optional, but recommended)

**Good News:**

- Deprecated aliases provide backward compatibility through 0.28+
- Most code continues to work with deprecation warnings
- Migration can be gradual

## Quick Migration Checklist

- [ ] Update `Cargo.toml`: `odos-sdk = "1.0"`
- [ ] Replace `OdosSor` → `OdosClient`
- [ ] Replace `SwapContext` → `AssemblyRequest`
- [ ] Replace `get_swap_quote()` → `quote()`
- [ ] Replace `build_base_transaction()` → `assemble()`
- [ ] Consider adopting `SwapBuilder` for new code
- [ ] Review feature flags if using contract bindings
- [ ] Run tests and address deprecation warnings

## Type Renames

### OdosSor → OdosClient

The main client type has been renamed for clarity.

**Before (0.x):**

```rust
use odos_sdk::OdosSor;

let client = OdosSor::new()?;
```

**After (1.0):**

```rust
use odos_sdk::OdosClient;

let client = OdosClient::new()?;
```

**Or use prelude:**

```rust
use odos_sdk::prelude::*;

let client = OdosClient::new()?;
```

### SwapContext → AssemblyRequest

The transaction assembly type has been renamed to better describe its purpose.

**Before (0.x):**

```rust
use odos_sdk::SwapContext;

let context = SwapContext::builder()
    .chain(chain)
    .router_address(router)
    .signer_address(signer)
    .output_recipient(recipient)
    .token_address(token)
    .token_amount(amount)
    .path_id(path_id)
    .build();

let tx = client.build_base_transaction(&context).await?;
```

**After (1.0):**

```rust
use odos_sdk::AssemblyRequest;

let request = AssemblyRequest::builder()
    .chain(chain)
    .router_address(router)
    .signer_address(signer)
    .output_recipient(recipient)
    .token_address(token)
    .token_amount(amount)
    .path_id(path_id)
    .build();

let tx = client.assemble(&request).await?;
```

## Method Renames

### get_swap_quote() → quote()

**Before (0.x):**

```rust
let quote = client.get_swap_quote(&quote_request).await?;
```

**After (1.0):**

```rust
let quote = client.quote(&quote_request).await?;
```

### build_base_transaction() → assemble()

**Before (0.x):**

```rust
let tx = client.build_base_transaction(&context).await?;
```

**After (1.0):**

```rust
let tx = client.assemble(&assembly_request).await?;
```

## New High-Level API: SwapBuilder

The biggest improvement in 1.0 is the new `SwapBuilder` API, which dramatically simplifies common swap operations.

### Before: Manual Quote + Assemble (0.x)

```rust
use odos_sdk::{OdosSor, QuoteRequest, SwapContext};
use alloy_chains::NamedChain;

let client = OdosSor::new()?;

// Step 1: Build quote request
let quote_request = QuoteRequest::builder()
    .chain_id(1)
    .input_tokens(vec![(usdc, amount).into()])
    .output_tokens(vec![(weth, 1).into()])
    .slippage_limit_percent(0.5)
    .user_addr(my_address.to_string())
    .compact(false)
    .simple(false)
    .referral_code(0)
    .disable_rfqs(false)
    .build();

// Step 2: Get quote
let quote = client.get_swap_quote(&quote_request).await?;

// Step 3: Build swap context
let context = SwapContext::builder()
    .chain(NamedChain::Mainnet)
    .router_address(NamedChain::Mainnet.v2_router_address()?)
    .signer_address(my_address)
    .output_recipient(my_address)
    .token_address(usdc)
    .token_amount(amount)
    .path_id(quote.path_id().to_string())
    .build();

// Step 4: Build transaction
let tx = client.build_base_transaction(&context).await?;
```

### After: SwapBuilder (1.0 - Recommended)

```rust
use odos_sdk::prelude::*;

let client = OdosClient::new()?;

let tx = client.swap()
    .chain(Chain::ethereum())
    .from_token(usdc, amount)
    .to_token(weth)
    .slippage(Slippage::percent(0.5)?)
    .signer(my_address)
    .build_transaction()
    .await?;
```

**15+ lines reduced to 5 lines!**

### SwapBuilder Features

The new API includes type-safe domain types:

```rust
use odos_sdk::prelude::*;

// Type-safe chains
let chain = Chain::ethereum();      // or arbitrum(), base(), etc.
let chain = Chain::from_chain_id(42161)?;

// Type-safe slippage
let slippage = Slippage::percent(0.5)?;     // 0.5%
let slippage = Slippage::bps(50)?;          // 50 basis points
let slippage = Slippage::standard();         // 0.5% (common default)

// Type-safe referral codes
let referral = ReferralCode::new(12345);
let referral = ReferralCode::NONE;

// Build swap
let tx = client.swap()
    .chain(chain)
    .from_token(usdc, amount)
    .to_token(weth)
    .slippage(slippage)
    .signer(my_address)
    .referral(referral)
    .build_transaction()
    .await?;
```

### Quote-First Workflow with SwapBuilder

```rust
use odos_sdk::prelude::*;

// Get quote to show user expected output
let quote = client.swap()
    .chain(Chain::ethereum())
    .from_token(usdc, amount)
    .to_token(weth)
    .slippage(Slippage::percent(0.5)?)
    .signer(my_address)
    .quote()
    .await?;

println!("You will receive approximately: {}", quote.out_amount());

// User confirms, build transaction with same builder
let tx = client.swap()
    .chain(Chain::ethereum())
    .from_token(usdc, amount)
    .to_token(weth)
    .slippage(Slippage::percent(0.5)?)
    .signer(my_address)
    .build_transaction()
    .await?;
```

## Feature Flags

If you're using contract bindings, 1.0 introduces granular feature flags.

### Default Configuration

**Before (0.x):**

```toml
[dependencies]
odos-sdk = "1.0"
```

**After (1.0):**

```toml
[dependencies]
odos-sdk = "1.0"  # Same default: includes v2 + v3 routers
```

### Minimal Configuration (API only)

**New in 1.0:**

```toml
[dependencies]
odos-sdk = { version = "1.0", default-features = false, features = ["minimal"] }
```

Use this if you only need the HTTP API client without contract bindings. Reduces compile time and dependencies.

### Specific Router Versions

**New in 1.0:**

```toml
# Only V2 router
[dependencies]
odos-sdk = { version = "1.0", default-features = false, features = ["v2"] }

# Only V3 router (includes v2 for SwapInputs type)
[dependencies]
odos-sdk = { version = "1.0", default-features = false, features = ["v3"] }

# All routers including limit orders
[dependencies]
odos-sdk = { version = "1.0", default-features = false, features = ["contracts"] }
```

## Import Simplification: Prelude

1.0 introduces a prelude module for convenient imports.

**Before (0.x):**

```rust
use odos_sdk::{OdosSor, QuoteRequest, SwapContext};
use alloy_primitives::{Address, U256};
```

**After (1.0):**

```rust
use odos_sdk::prelude::*;
// Imports: OdosClient, QuoteRequest, AssemblyRequest, Chain, Slippage,
//          ReferralCode, OdosError, Result, Address, U256
```

## Error Handling Improvements

1.0 adds convenience methods for error handling.

**New in 1.0:**

```rust
use odos_sdk::prelude::*;

match client.quote(&request).await {
    Ok(quote) => { /* ... */ }
    Err(e) => {
        // New convenience methods
        if e.is_client_error() {
            eprintln!("Invalid request (4xx)");
        } else if e.is_server_error() {
            eprintln!("Server error (5xx)");
        }

        // Suggested retry delay
        if let Some(delay) = e.suggested_retry_delay() {
            tokio::time::sleep(delay).await;
        }

        // Existing error code checks still work
        if let Some(code) = e.error_code() {
            if code.is_no_viable_path() {
                eprintln!("No routing path found");
            }
        }
    }
}
```

## Step-by-Step Migration Example

Let's migrate a complete example.

### Before (0.x)

```rust
use odos_sdk::{OdosSor, QuoteRequest, SwapContext};
use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OdosSor::new()?;

    let usdc: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;
    let weth: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse()?;
    let my_address: Address = "0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0".parse()?;
    let amount = U256::from(1_000_000);

    // Get quote
    let quote_request = QuoteRequest::builder()
        .chain_id(1)
        .input_tokens(vec![(usdc, amount).into()])
        .output_tokens(vec![(weth, 1).into()])
        .slippage_limit_percent(0.5)
        .user_addr(my_address.to_string())
        .compact(false)
        .simple(false)
        .referral_code(0)
        .disable_rfqs(false)
        .build();

    let quote = client.get_swap_quote(&quote_request).await?;
    println!("Expected output: {}", quote.out_amount().unwrap());

    // Build transaction
    let context = SwapContext::builder()
        .chain(NamedChain::Mainnet)
        .router_address(NamedChain::Mainnet.v2_router_address()?)
        .signer_address(my_address)
        .output_recipient(my_address)
        .token_address(usdc)
        .token_amount(amount)
        .path_id(quote.path_id().to_string())
        .build();

    let tx = client.build_base_transaction(&context).await?;
    println!("Transaction ready!");

    Ok(())
}
```

### After (1.0 - Option 1: Minimal Changes)

Replace names but keep the same structure:

```rust
use odos_sdk::{OdosClient, QuoteRequest, AssemblyRequest};  // Changed names
use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OdosClient::new()?;  // Changed

    let usdc: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;
    let weth: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse()?;
    let my_address: Address = "0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0".parse()?;
    let amount = U256::from(1_000_000);

    // Get quote
    let quote_request = QuoteRequest::builder()
        .chain_id(1)
        .input_tokens(vec![(usdc, amount).into()])
        .output_tokens(vec![(weth, 1).into()])
        .slippage_limit_percent(0.5)
        .user_addr(my_address)  // Now accepts Address directly
        .compact(false)
        .simple(false)
        .referral_code(0)
        .disable_rfqs(false)
        .build();

    let quote = client.quote(&quote_request).await?;  // Changed method name
    println!("Expected output: {}", quote.out_amount().unwrap());

    // Build transaction
    let request = AssemblyRequest::builder()  // Changed name
        .chain(NamedChain::Mainnet)
        .router_address(NamedChain::Mainnet.v2_router_address()?)
        .signer_address(my_address)
        .output_recipient(my_address)
        .token_address(usdc)
        .token_amount(amount)
        .path_id(quote.path_id().to_string())
        .build();

    let tx = client.assemble(&request).await?;  // Changed method name
    println!("Transaction ready!");

    Ok(())
}
```

### After (1.0 - Option 2: SwapBuilder - Recommended)

Use the new high-level API:

```rust
use odos_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OdosClient::new()?;

    let usdc: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;
    let weth: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse()?;
    let my_address: Address = "0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0".parse()?;
    let amount = U256::from(1_000_000);

    // Get quote
    let quote = client.swap()
        .chain(Chain::ethereum())
        .from_token(usdc, amount)
        .to_token(weth)
        .slippage(Slippage::percent(0.5)?)
        .signer(my_address)
        .quote()
        .await?;

    println!("Expected output: {}", quote.out_amount().unwrap());

    // Build transaction
    let tx = client.swap()
        .chain(Chain::ethereum())
        .from_token(usdc, amount)
        .to_token(weth)
        .slippage(Slippage::percent(0.5)?)
        .signer(my_address)
        .build_transaction()
        .await?;

    println!("Transaction ready!");

    Ok(())
}
```

**Result: 32 lines → 24 lines, much clearer intent!**

## Testing Your Migration

After migrating, run these checks:

```bash
# 1. Ensure it compiles
cargo build

# 2. Run tests
cargo test

# 3. Check for warnings
cargo clippy --all-targets --all-features -- -D warnings

# 4. Test documentation examples
cargo test --doc

# 5. Check documentation builds
cargo doc --all-features --no-deps
```

## Deprecation Timeline

- **0.27.x**: Current stable release
- **0.28.x**: Introduces deprecation warnings for old names (planned)
- **1.0.0**: Removes deprecated aliases, new names become standard

## Need Help?

- **Examples**: See [EXAMPLES.md](EXAMPLES.md) for comprehensive patterns
- **Getting Started**: Check [GETTING_STARTED.md](GETTING_STARTED.md) for tutorials
- **Issues**: Report problems at [GitHub Issues](https://github.com/semiotic-ai/odos-sdk/issues)
- **Odos Docs**: [docs.odos.xyz](https://docs.odos.xyz)

## Summary

**Key Changes:**

1. ✅ `OdosSor` → `OdosClient`
2. ✅ `SwapContext` → `AssemblyRequest`
3. ✅ `get_swap_quote()` → `quote()`
4. ✅ `build_base_transaction()` → `assemble()`
5. ✨ New `SwapBuilder` API (recommended)
6. ✨ Type-safe `Chain`, `Slippage`, `ReferralCode`
7. ✨ Convenient `prelude` module
8. ✨ Granular feature flags

**Migration Strategy:**

- **Gradual**: Update names, keep existing patterns (Option 1)
- **Modern**: Adopt `SwapBuilder` for cleaner code (Option 2)
- **Both**: Mix approaches - use `SwapBuilder` for new code, update names in old code

The 1.0 API is designed to be more intuitive, type-safe, and maintainable. We believe these changes will make your code clearer and more robust.

Happy swapping!
