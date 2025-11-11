# Odos Rust SDK

[![Crates.io](https://img.shields.io/crates/v/odos-sdk.svg)](https://crates.io/crates/odos-sdk)
[![Crates.io Downloads](https://img.shields.io/crates/d/odos-sdk.svg)](https://crates.io/crates/odos-sdk)
[![Documentation](https://docs.rs/odos-sdk/badge.svg)](https://docs.rs/odos-sdk)
[![License](https://img.shields.io/crates/l/odos-sdk.svg)](https://github.com/semiotic-ai/odos-sdk/blob/main/LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.90%2B-blue.svg?logo=rust)](https://www.rust-lang.org)

A production-ready Rust SDK for [Odos](https://www.odos.xyz) - the decentralized exchange aggregator that finds optimal token swap routes across 16+ EVM chains. Built with type safety, reliability, and developer experience in mind.

## What Makes This Special

**Optimal pricing through advanced routing** - Odos analyzes paths across hundreds of DEXes and liquidity sources, splitting trades intelligently to minimize slippage and maximize output. The SDK makes this power accessible in just a few lines of Rust.

**Battle-tested for production:**

- Smart retry logic with exponential backoff for network resilience
- Structured error codes with clear categorization and trace IDs
- Rate limit detection with `Retry-After` support
- Full type safety via Alloy primitives (no string addresses or numeric guessing)
- Connection pooling, configurable timeouts, and graceful degradation
- Comprehensive logging and observability with `tracing`

**APIs for every use case:**

- High-level `SwapBuilder` - integrate swaps in minutes
- Mid-level `quote` → `assemble` - full control over the flow
- Low-level contract bindings - advanced scenarios and direct router access

## Quick Start

Add the SDK to your project:

```toml
[dependencies]
odos-sdk = "1.0"
```

### Your First Swap

```rust
use odos_sdk::prelude::*;
use alloy_primitives::{Address, U256};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the client
    let client = OdosClient::new()?;

    // Define your swap
    let usdc: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;
    let weth: Address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse()?;
    let my_address: Address = "0x...".parse()?;

    // Build the transaction - that's it!
    let tx = client.swap()
        .chain(Chain::ethereum())
        .from_token(usdc, U256::from(1_000_000)) // 1 USDC (6 decimals)
        .to_token(weth)
        .slippage(Slippage::percent(0.5)?)
        .signer(my_address)
        .build_transaction()
        .await?;

    // Execute with your wallet
    // provider.send_transaction(tx.transaction).await?;

    Ok(())
}
```

Three concepts, one builder, zero complexity. The SDK handles quote fetching, optimal routing, transaction assembly, and error recovery automatically.

**Next steps:** Check out [GETTING_STARTED.md](GETTING_STARTED.md) for a complete walkthrough, or jump to [EXAMPLES.md](EXAMPLES.md) for common patterns.

## Core Features

### Multi-Chain Support

Supports 16+ EVM chains out of the box:

| Category | Chains |
|----------|--------|
| **Layer 1** | Ethereum |
| **Layer 2** | Arbitrum, Optimism, Base, Polygon, zkSync, Scroll, Linea, Mantle, Mode |
| **Sidechains** | BSC, Avalanche, Fraxtal, Sonic, Unichain |

Chain selection is type-safe and simple:

```rust
let chain = Chain::ethereum();    // or arbitrum(), optimism(), etc.
let chain = Chain::from_chain_id(42161)?;  // From numeric ID
```

### Complete Type Safety

Built on the [Alloy](https://github.com/alloy-rs/alloy) ecosystem for bulletproof type safety:

```rust
use alloy_primitives::{Address, U256};

// Parse addresses at compile time or runtime
let token: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".parse()?;

// Handle amounts with U256 (no floating point errors)
let one_usdc = U256::from(1_000_000);  // 6 decimals

// Validated slippage
let slippage = Slippage::percent(0.5)?;  // 0.5%
let slippage = Slippage::bps(50)?;       // 50 basis points
```

### Resilient Error Handling

Structured errors with clear categorization:

```rust
use odos_sdk::{OdosError, error_code::OdosErrorCode};

match client.quote(&request).await {
    Ok(quote) => {
        println!("Expected output: {}", quote.out_amount());
    }
    Err(e) => {
        // Check error type
        if e.is_rate_limit() {
            if let Some(retry_after) = e.retry_after() {
                println!("Rate limited. Retry after: {:?}", retry_after);
            }
        }

        // Check specific error codes
        if let Some(code) = e.error_code() {
            if code.is_no_viable_path() {
                println!("No routing path found for this pair");
            } else if code.is_validation_error() {
                println!("Invalid request parameters");
            }
        }

        // Access trace ID for debugging
        if let Some(trace_id) = e.trace_id() {
            eprintln!("Error trace ID: {}", trace_id);
        }
    }
}
```

Error codes match the [Odos API documentation](https://docs.odos.xyz/build/api_errors) with type-safe categorization:

- **1XXX**: General API errors
- **2XXX**: Quote/routing errors (`NoViablePath`, `AlgoTimeout`, etc.)
- **3XXX**: Internal service errors
- **4XXX**: Validation errors (`InvalidChainId`, `InvalidTokenAmount`, etc.)
- **5XXX**: Internal errors

### Smart Retry Logic

Configurable retry behavior with exponential backoff:

```rust
use std::time::Duration;
use odos_sdk::{OdosClient, RetryConfig};

// Conservative preset - only retry network errors
let client = OdosClient::with_retry_config(RetryConfig::conservative())?;

// Custom retry configuration
let client = OdosClient::with_retry_config(RetryConfig {
    max_retries: 5,
    initial_backoff_ms: 200,
    retry_server_errors: true,
    retry_predicate: Some(|err| {
        // Custom logic to determine if an error should be retried
        err.is_timeout() || (err.is_api_error() && !err.is_validation_error())
    }),
})?;
```

Rate limits are detected but **not** automatically retried - you control the global rate limiting strategy.

## Three Ways to Swap

The SDK provides three levels of abstraction. Choose based on your needs:

### 1. High-Level: SwapBuilder

Perfect for most use cases. One builder, automatic flow:

```rust
use odos_sdk::prelude::*;

let tx = client.swap()
    .chain(Chain::arbitrum())
    .from_token(usdc, amount)
    .to_token(weth)
    .slippage(Slippage::percent(0.5)?)
    .signer(my_address)
    .recipient(recipient_address)  // Optional: send output to different address
    .build_transaction()
    .await?;

// Or just get a quote first
let quote = client.swap()
    .chain(Chain::ethereum())
    .from_token(usdc, amount)
    .to_token(weth)
    .slippage(Slippage::percent(0.5)?)
    .signer(my_address)
    .quote()
    .await?;

println!("Expected output: {}", quote.out_amount());
```

### 2. Mid-Level: Quote + Assemble

More control over the quote and assembly phases:

```rust
use odos_sdk::prelude::*;

// Step 1: Request quote
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

// Show user the expected output
println!("You will receive approximately: {}", quote.out_amount());

// Step 2: User confirms, assemble transaction
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

### 3. Low-Level: Contract Bindings

Direct router contract access for advanced scenarios:

```rust
use odos_sdk::{OdosV2Router, OdosChain};
use alloy_chains::NamedChain;

let router_address = NamedChain::Mainnet.v2_router_address()?;
let router = OdosV2Router::new(router_address, provider);

// Call contract methods directly
let result = router.swap(swap_inputs).send().await?;
```

## Configuration

### Basic Configuration

```rust
use odos_sdk::{OdosClient, ClientConfig};
use std::time::Duration;

let config = ClientConfig {
    timeout: Duration::from_secs(30),
    connect_timeout: Duration::from_secs(10),
    max_connections: 20,
    pool_idle_timeout: Duration::from_secs(90),
    ..Default::default()
};

let client = OdosClient::with_config(config)?;
```

### API Endpoints

Choose between public and enterprise endpoints:

```rust
use odos_sdk::{ClientConfig, Endpoint, ApiHost, ApiVersion};

// Public API (default)
let config = ClientConfig {
    endpoint: Endpoint::public_v2(),  // or public_v3()
    ..Default::default()
};

// Enterprise API with higher rate limits
let config = ClientConfig {
    endpoint: Endpoint::enterprise_v2(),  // or enterprise_v3()
    api_key: Some(ApiKey::new("your-api-key")?),
    ..Default::default()
};

let client = OdosClient::with_config(config)?;
```

### Feature Flags

Customize what gets compiled based on your needs:

```toml
# Default: V2 + V3 routers
[dependencies]
odos-sdk = "1.0"

# Minimal: Just API types and HTTP client (no contract bindings)
[dependencies]
odos-sdk = { version = "1.0", default-features = false, features = ["minimal"] }

# All contracts: V2 + V3 + Limit Orders
[dependencies]
odos-sdk = { version = "1.0", default-features = false, features = ["contracts"] }

# Custom combination
[dependencies]
odos-sdk = { version = "1.0", default-features = false, features = ["v2", "v3"] }
```

Available features:

- `minimal` - Core API types and HTTP client only
- `v2` - V2 router contract bindings
- `v3` - V3 router contract bindings (includes v2)
- `limit-orders` - Limit order contract bindings (includes v2)
- `contracts` - All contract bindings (v2 + v3 + limit-orders)
- `default` - V2 + V3 routers

## Documentation

| Resource | Description |
|----------|-------------|
| [GETTING_STARTED.md](GETTING_STARTED.md) | Complete walkthrough from setup to your first swap |
| [EXAMPLES.md](EXAMPLES.md) | Real-world patterns: error handling, testing, integration |
| [API Docs](https://docs.rs/odos-sdk) | Complete API reference with inline examples |
| [ERROR_HANDLING_GUIDE.md](ERROR_HANDLING_GUIDE.md) | Deep dive into error types and recovery strategies |
| [CHANGELOG.md](CHANGELOG.md) | Version history and migration guides |
| [SECURITY.md](SECURITY.md) | Security best practices and vulnerability reporting |

## Advanced Topics

### Rate Limiting Strategy

The SDK detects rate limits but doesn't retry them automatically. Implement your own strategy:

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn quote_with_rate_limiting(
    client: &OdosClient,
    request: &QuoteRequest,
) -> Result<SingleQuoteResponse, OdosError> {
    loop {
        match client.quote(request).await {
            Ok(quote) => return Ok(quote),
            Err(e) if e.is_rate_limit() => {
                let wait = e.retry_after()
                    .unwrap_or(Duration::from_secs(5));
                eprintln!("Rate limited, waiting {:?}", wait);
                sleep(wait).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

For production applications with high request volumes:

1. Share a single `OdosClient` instance across your application
2. Implement a token bucket or leaky bucket algorithm
3. Consider using a rate limiting library like `governor`
4. Monitor rate limit errors and adjust your request rate dynamically

### Router Versioning

The SDK supports multiple router versions with different capabilities:

```rust
use odos_sdk::{OdosChain, RouterAvailability};
use alloy_chains::NamedChain;

let chain = NamedChain::Mainnet;
let availability = chain.router_availability()?;

if availability.v3 {
    // Use V3 router (unified address across all chains)
    let router = chain.v3_router_address()?;
} else if availability.v2 {
    // Fall back to V2 router
    let router = chain.v2_router_address()?;
}

if availability.limit_order {
    // Limit orders are supported
    let lo_router = chain.lo_router_address()?;
}
```

V3 router features:

- Deployed at the same address on all supported chains (CREATE2)
- Enhanced gas efficiency
- Improved MEV protection

### Chain Support Detection

Check if a chain is supported before attempting operations:

```rust
use odos_sdk::OdosChain;
use alloy_chains::NamedChain;

let chain = NamedChain::Mainnet;

// Check general Odos support
if chain.supports_odos() {
    println!("Odos is available on this chain");
}

// Check specific router availability
let availability = chain.router_availability()?;
println!("V2: {}, V3: {}, Limit Orders: {}",
    availability.v2,
    availability.v3,
    availability.limit_order
);
```

## Examples

See [EXAMPLES.md](EXAMPLES.md) for comprehensive examples including:

- Multi-token swaps
- Error recovery and retry strategies
- Integration with wallets (ethers-rs, foundry)
- Gas estimation and optimization
- Testing with mocks
- Cross-chain workflows
- Production deployment patterns

Quick example - handling errors gracefully:

```rust
use odos_sdk::prelude::*;

async fn robust_swap(
    client: &OdosClient,
    from: Address,
    to: Address,
    amount: U256,
) -> Result<AssemblyResponse, OdosError> {
    let result = client.swap()
        .chain(Chain::ethereum())
        .from_token(from, amount)
        .to_token(to)
        .slippage(Slippage::percent(0.5)?)
        .signer(my_address)
        .build_transaction()
        .await;

    match result {
        Ok(tx) => Ok(tx),
        Err(e) if e.is_no_viable_path() => {
            eprintln!("No routing path available - try increasing slippage or different tokens");
            Err(e)
        }
        Err(e) if e.is_timeout() => {
            eprintln!("Request timed out - network or service issue");
            Err(e)
        }
        Err(e) if e.is_rate_limit() => {
            eprintln!("Rate limited - implement backoff strategy");
            Err(e)
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
            if let Some(trace_id) = e.trace_id() {
                eprintln!("Trace ID for support: {}", trace_id);
            }
            Err(e)
        }
    }
}
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for:

- Development setup (Rust 1.90+)
- Code standards and formatting
- Testing requirements
- PR process
- Release workflow

Quick development commands:

```bash
# Build
cargo build

# Run tests
cargo test

# Lint (CI enforces zero warnings)
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt

# Security audit
cargo audit
```

## Support

- **Issues**: [GitHub Issues](https://github.com/semiotic-ai/odos-sdk/issues)
- **Odos Documentation**: [docs.odos.xyz](https://docs.odos.xyz)
- **Odos Discord**: [discord.gg/odos](https://discord.gg/odos)

## Security

Security is a top priority. Please report vulnerabilities to <security@semiotic.ai>. See [SECURITY.md](SECURITY.md) for:

- Vulnerability reporting process
- API key security best practices
- Input validation guidelines
- Production deployment checklist

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

Built with the excellent [Alloy](https://github.com/alloy-rs/alloy) ecosystem for Ethereum interactions.

---

**Ready to integrate?** Start with [GETTING_STARTED.md](GETTING_STARTED.md) or dive into [EXAMPLES.md](EXAMPLES.md) for practical patterns.
