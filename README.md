# Odos Rust SDK

A [Rust](https://www.rust-lang.org/) SDK for [Odos](https://docs.odos.xyz/)

[![Crates.io](https://img.shields.io/crates/v/odos-sdk.svg)](https://crates.io/crates/odos-sdk)
[![Crates.io Downloads](https://img.shields.io/crates/d/odos-sdk.svg)](https://crates.io/crates/odos-sdk)
[![License](https://img.shields.io/crates/l/odos-sdk.svg)](https://github.com/semiotic-ai/odos-sdk/blob/main/LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-2021-blue.svg?logo=rust)](https://www.rust-lang.org)
[![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)](https://github.com/semiotic-ai/odos-sdk/)

---

## Features

This SDK provides granular feature flags to minimize dependencies and compile times:

- **`default`** - Most common use case with V2 and V3 router support (`v2` + `v3`)
- **`minimal`** - Core API types and HTTP client only (no contract bindings)
- **`v2`** - V2 router contract bindings (base feature for all contract types)
- **`v3`** - V3 router contract bindings (includes `v2`)
- **`limit-orders`** - Limit order contract bindings (includes `v2`)
- **`contracts`** - All contract bindings (`v2` + `v3` + `limit-orders`)

### Usage

```toml
# Default features (v2 + v3)
[dependencies]
odos-sdk = "0.25"

# Minimal - API client only, no contract bindings
[dependencies]
odos-sdk = { version = "0.25", default-features = false, features = ["minimal"] }

# Only V2 router support
[dependencies]
odos-sdk = { version = "0.25", default-features = false, features = ["v2"] }

# All contract bindings
[dependencies]
odos-sdk = { version = "0.25", default-features = false, features = ["contracts"] }
```

**Note:** The `v2` feature is the base contract feature that provides the `SwapInputs` type used by all router implementations. Both `v3` and `limit-orders` features automatically enable `v2`.

---

## One-to-one Swap Example

> **Note**: This example demonstrates the workflow for executing a token swap using the Odos SDK.
> It assumes you have set up an alloy provider and signer. For a complete working example,
> see the [documentation](https://docs.rs/odos-sdk).

```rust,no_run
use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};
use odos_sdk::{OdosSor, QuoteRequest, SwapContext};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# // Setup (not shown): Initialize your alloy provider and signer
# // let provider = ...; // Your alloy provider
# // let signer = ...; // Your alloy signer with private key
# let signer_address = Address::ZERO; // Replace with actual signer address
# let balance = U256::from(1000000000000000000u64); // Example: 1 token
#
// Token addresses
const WETH: &str = "0x4200000000000000000000000000000000000006";
const USDC: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";
const RECIPIENT: &str = "0x83384D138420436f4b0DaE97b02002dd5011a7D9";

let chain = NamedChain::Base;
let weth_address = WETH.parse::<Address>()?;

// Step 1: Get a quote from Odos
let quote_request = QuoteRequest::builder()
    .chain_id(chain.into())
    .input_tokens(vec![(weth_address, balance).into()])
    .output_tokens(vec![(USDC.parse::<Address>()?, 1).into()])
    .slippage_limit_percent(1.0)
    .user_addr(signer_address.to_string())
    .compact(false)
    .simple(false)
    .referral_code(0)
    .disable_rfqs(true)
    .build();

let sor_client = OdosSor::new()?;
let quote = sor_client.get_swap_quote(&quote_request).await?;

// Step 2: Approve the Odos router to spend your tokens
// (Using your ERC20 contract implementation)
// let router_address = chain.v2_router_address()?;
// token_contract.approve(router_address, balance).await?;

// Step 3: Build the swap transaction
let swap_params = SwapContext::builder()
    .chain(chain)
    .router_address(chain.v2_router_address()?)
    .signer_address(signer_address)
    .output_recipient(RECIPIENT.parse::<Address>()?)
    .token_address(weth_address)
    .token_amount(balance)
    .path_id(quote.path_id().to_string())
    .build();

let tx_request = sor_client
    .build_base_transaction(&swap_params)
    .await?;

// Step 4: Send the transaction
// (Using your alloy signer)
// let pending_tx = signer.send_transaction(tx_request).await?;
// let receipt = pending_tx.get_receipt().await?;
// assert!(receipt.status());

println!("Swap transaction built successfully!");
# Ok(())
# }
```
