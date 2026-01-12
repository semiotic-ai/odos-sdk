# Getting Started with Odos SDK

This guide walks you through integrating the Odos SDK into your Rust project, from installation to executing your first token swap.

## Table of Contents

1. [Installation](#installation)
2. [Core Concepts](#core-concepts)
3. [Your First Swap](#your-first-swap)
4. [Understanding Quotes](#understanding-quotes)
5. [Executing Transactions](#executing-transactions)
6. [Error Handling](#error-handling)
7. [Next Steps](#next-steps)

## Installation

Add the SDK to your `Cargo.toml`:

```toml
[dependencies]
odos-sdk = "2.0"
alloy = { version = "1.3", features = ["full"] }  # For wallet/signer functionality
tokio = { version = "1", features = ["full"] }
```

### Feature Flags

The SDK offers granular feature flags to minimize dependencies:

```toml
# Default: V2 + V3 routers (recommended)
odos-sdk = "2.0"

# Minimal: API client only, no contract bindings
odos-sdk = { version = "2.0", default-features = false, features = ["minimal"] }

# All contracts: V2 + V3 + Limit Orders
odos-sdk = { version = "2.0", default-features = false, features = ["contracts"] }

# Specific router versions
odos-sdk = { version = "2.0", default-features = false, features = ["v2"] }
odos-sdk = { version = "2.0", default-features = false, features = ["v3"] }
```

## Core Concepts

### 1. The Client

`OdosClient` is your entry point to the SDK. Create one instance and share it across your application:

```rust
use odos_sdk::OdosClient;

let client = OdosClient::new()?;
```

### 2. Chains

The SDK supports 16+ EVM chains. Use the type-safe `Chain` wrapper:

```rust
use odos_sdk::Chain;

let ethereum = Chain::ethereum();
let arbitrum = Chain::arbitrum();
let base = Chain::base();
let polygon = Chain::polygon();
```

### 3. Tokens and Amounts

All token addresses and amounts use Alloy primitives for type safety:

```rust
use alloy_primitives::{Address, U256};

// Parse address from string
let usdc: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;

// Create amounts (6 decimals for USDC)
let amount = U256::from(1_000_000); // 1 USDC
```

### 4. Slippage

Slippage protection is type-safe and validated:

```rust
use odos_sdk::Slippage;

let slippage = Slippage::percent(0.5)?;  // 0.5%
let slippage = Slippage::bps(50)?;       // 50 basis points = 0.5%
```

## Your First Swap

Let's build a complete example that swaps USDC for WETH on Ethereum.

### Step 1: Setup

```rust
use odos_sdk::prelude::*;
use alloy_primitives::{Address, U256};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    // Create client
    let client = OdosClient::new()?;

    // Define tokens
    let usdc = Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?;
    let weth = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")?;

    // Your wallet address
    let my_address = Address::from_str("0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0")?;

    // Amount to swap (1 USDC = 1,000,000 with 6 decimals)
    let amount = U256::from(1_000_000);

    // Continue to next step...
    Ok(())
}
```

### Step 2: Build the Swap

Using the high-level `SwapBuilder` API:

```rust
let tx = client.swap()
    .chain(Chain::ethereum())
    .from_token(usdc, amount)
    .to_token(weth)
    .slippage(Slippage::percent(0.5)?)
    .signer(my_address)
    .build_transaction()
    .await?;

println!("Transaction ready!");
println!("To: {:?}", tx.to);
println!("Data: {:?}", tx.input);
```

### Step 3: Execute (with Alloy)

```rust
use alloy::{
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    network::TransactionBuilder,
};

// Setup provider and signer
let rpc_url = "https://eth.llamarpc.com".parse()?;
let provider = ProviderBuilder::new().on_http(rpc_url);

let signer = PrivateKeySigner::from_bytes(&your_private_key)?;
let wallet = alloy::signers::EthereumWallet::from(signer);

// Send transaction
let pending_tx = provider.send_transaction(tx).await?;
println!("Transaction sent: {}", pending_tx.tx_hash());

// Wait for confirmation
let receipt = pending_tx.get_receipt().await?;
println!("Transaction confirmed in block {}", receipt.block_number.unwrap());
```

## Understanding Quotes

Before executing a swap, you typically want to show users the expected output. Use the quote workflow:

```rust
use odos_sdk::{QuoteRequest, OdosClient};

let client = OdosClient::new()?;

// Build quote request
let quote_request = QuoteRequest::builder()
    .chain_id(1)  // Ethereum
    .input_tokens(vec![(usdc, amount).into()])
    .output_tokens(vec![(weth, 1).into()])  // '1' means variable output
    .slippage_limit_percent(0.5)
    .user_addr(my_address)
    .compact(false)
    .simple(false)
    .referral_code(0)
    .disable_rfqs(false)
    .build();

// Get quote
let quote = client.quote(&quote_request).await?;

// Display to user
println!("Input: {} USDC", quote.in_amount().unwrap());
println!("Expected output: {} WETH", quote.out_amount().unwrap());
println!("Gas estimate: {}", quote.gas_estimate());
println!("Price impact: {}%", quote.price_impact());
```

### Quote Parameters Explained

- **input_tokens**: Tokens you're swapping from (address, amount)
- **output_tokens**: Tokens you're swapping to (address, min amount or 1 for variable)
- **slippage_limit_percent**: Maximum acceptable slippage (0.5 = 0.5%)
- **user_addr**: Your wallet address (used for simulation)
- **compact**: Set to `false` for detailed quote information
- **simple**: Set to `false` for multi-hop routes
- **referral_code**: Optional referral code (use 0 for none)
- **disable_rfqs**: Set to `true` to disable Request-for-Quote sources

## Executing Transactions

After getting a quote, you need to assemble and execute the transaction.

### Option 1: SwapBuilder (Recommended)

The `SwapBuilder` handles both quote and assembly in one step:

```rust
let tx = client.swap()
    .chain(Chain::ethereum())
    .from_token(usdc, amount)
    .to_token(weth)
    .slippage(Slippage::percent(0.5)?)
    .signer(my_address)
    .build_transaction()
    .await?;
```

### Option 2: Manual Quote + Assembly

For more control, separate the steps:

```rust
use odos_sdk::{QuoteRequest, AssemblyRequest};
use alloy_chains::NamedChain;

// Step 1: Get quote
let quote_request = QuoteRequest::builder()
    .chain_id(1)
    .input_tokens(vec![(usdc, amount).into()])
    .output_tokens(vec![(weth, 1).into()])
    .slippage_limit_percent(0.5)
    .user_addr(my_address)
    .compact(false)
    .simple(false)
    .referral_code(0)
    .disable_rfqs(false)
    .build();

let quote = client.quote(&quote_request).await?;

// Step 2: Assemble transaction
let assembly_request = AssemblyRequest::builder()
    .chain(NamedChain::Mainnet)
    .router_address(NamedChain::Mainnet.v2_router_address()?)
    .signer_address(my_address)
    .output_recipient(my_address)
    .token_address(usdc)
    .token_amount(amount)
    .path_id(quote.path_id().to_string())
    .build();

let tx = client.assemble(&assembly_request).await?;
```

### Before Executing: Token Approval

Before the router can swap your tokens, you need to approve it:

```rust
use alloy::contract::SolCall;
use alloy_primitives::U256;

// ERC20 approval
let erc20 = IERC20::new(usdc, provider);
let router_address = NamedChain::Mainnet.v2_router_address()?;

// Approve router to spend your tokens
let approval_tx = erc20.approve(router_address, amount);
let pending = approval_tx.send().await?;
let receipt = pending.get_receipt().await?;

println!("Approval confirmed!");
```

## Error Handling

The SDK provides comprehensive error handling with structured error codes.

### Basic Error Handling

```rust
use odos_sdk::{OdosError, Result};

match client.quote(&quote_request).await {
    Ok(quote) => {
        println!("Quote received: {}", quote.path_id());
    }
    Err(e) => {
        eprintln!("Error: {}", e);

        // Check error category
        match e {
            OdosError::Api { status, message, .. } => {
                eprintln!("API error {}: {}", status, message);
            }
            OdosError::Http(err) => {
                eprintln!("Network error: {}", err);
            }
            OdosError::Timeout(_) => {
                eprintln!("Request timed out");
            }
            _ => {
                eprintln!("Other error: {}", e);
            }
        }
    }
}
```

### Structured Error Codes

Access specific error codes for fine-grained handling:

```rust
use odos_sdk::error_code::OdosErrorCode;

if let Err(e) = client.quote(&quote_request).await {
    if let Some(code) = e.error_code() {
        // Check error categories
        if code.is_validation_error() {
            eprintln!("Invalid request parameters");
        } else if code.is_no_viable_path() {
            eprintln!("No routing path found for this swap");
        } else if code.is_timeout() {
            eprintln!("Service timeout - try again");
        }

        // Check if retryable
        if code.is_retryable() {
            eprintln!("This error can be retried");
        }
    }

    // Get trace ID for support
    if let Some(trace_id) = e.trace_id() {
        eprintln!("Trace ID for support: {}", trace_id);
    }
}
```

### Rate Limiting

Handle rate limits gracefully:

```rust
use std::time::Duration;

match client.quote(&quote_request).await {
    Err(e) if e.is_rate_limit() => {
        eprintln!("Rate limited!");

        // Use Retry-After header if available
        if let Some(retry_after) = e.retry_after() {
            eprintln!("Retry after {} seconds", retry_after.as_secs());
            tokio::time::sleep(retry_after).await;
        } else {
            // Default backoff
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
    result => result?,
}
```

## Complete Example

Here's a complete, production-ready example:

```rust
use odos_sdk::prelude::*;
use alloy_primitives::{Address, U256};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize client with custom config
    let client = OdosClient::with_retry_config(RetryConfig {
        max_retries: 3,
        initial_backoff_ms: 100,
        retry_server_errors: true,
        retry_predicate: None,
    })?;

    // Token addresses (Ethereum mainnet)
    let usdc = Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?;
    let weth = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")?;
    let my_address = Address::from_str("0x742d35Cc6634C0532925a3b8D35f3e7a5edD29c0")?;

    // Amount (1 USDC)
    let amount = U256::from(1_000_000);

    println!("Getting quote...");

    // Build swap transaction
    let tx = match client.swap()
        .chain(Chain::ethereum())
        .from_token(usdc, amount)
        .to_token(weth)
        .slippage(Slippage::percent(0.5)?)
        .signer(my_address)
        .build_transaction()
        .await
    {
        Ok(tx) => tx,
        Err(e) if e.is_rate_limit() => {
            eprintln!("Rate limited. Please try again later.");
            if let Some(retry_after) = e.retry_after() {
                eprintln!("Retry after {} seconds", retry_after.as_secs());
            }
            return Err(e);
        }
        Err(e) => {
            eprintln!("Failed to build transaction: {}", e);
            if let Some(trace_id) = e.trace_id() {
                eprintln!("Trace ID: {}", trace_id);
            }
            return Err(e);
        }
    };

    println!("Transaction ready!");
    println!("To: {:?}", tx.to);
    println!("Value: {:?}", tx.value);
    println!("Gas limit: {:?}", tx.gas);

    // Execute with your wallet/signer
    // let pending = provider.send_transaction(tx).await?;
    // let receipt = pending.get_receipt().await?;

    Ok(())
}
```

## Next Steps

Now that you understand the basics, explore:

1. **[Examples](EXAMPLES.md)** - Advanced patterns and real-world scenarios
   - Multi-token swaps
   - Custom slippage strategies
   - Error recovery patterns
   - Integration with wallets
   - Testing strategies

2. **[API Documentation](https://docs.rs/odos-sdk)** - Complete API reference
   - All available methods
   - Configuration options
   - Advanced features

3. **Configuration**
   - [Custom retry strategies](https://docs.rs/odos-sdk/latest/odos_sdk/struct.RetryConfig.html)
   - [Client configuration](https://docs.rs/odos-sdk/latest/odos_sdk/struct.ClientConfig.html)
   - [Enterprise API setup](README.md#enterprise-api)

4. **Production Considerations**
   - Rate limiting strategies
   - Error monitoring and alerting
   - Gas optimization
   - Transaction monitoring

## Common Issues

### "No viable path found"

This error occurs when Odos can't find a route for your swap. Common causes:

- Insufficient liquidity for the token pair
- Token not supported on the selected chain
- Amount too large for available liquidity

**Solution**: Try a smaller amount or different token pair.

### "Rate limited"

You're making too many requests too quickly.

**Solution**:

- Share a single `OdosClient` across your application
- Implement application-level rate limiting
- Handle rate limit errors with proper backoff

### "Invalid chain ID"

The chain you specified isn't supported by Odos.

**Solution**: Check supported chains with `get_supported_chains()` or use the type-safe `Chain` constructors.

### Compilation errors with features

Make sure you have the correct feature flags enabled:

```toml
# For V2 router
odos-sdk = { version = "2.0", features = ["v2"] }

# For V3 router
odos-sdk = { version = "2.0", features = ["v3"] }

# For all features
odos-sdk = { version = "2.0", features = ["contracts"] }
```

## Support

- **GitHub Issues**: [github.com/semiotic-ai/odos-sdk/issues](https://github.com/semiotic-ai/odos-sdk/issues)
- **Odos Documentation**: [docs.odos.xyz](https://docs.odos.xyz)
- **Discord**: Join the Odos community for support

Happy swapping!
