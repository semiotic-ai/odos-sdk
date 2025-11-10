# Odos SDK Examples

This guide contains real-world examples and patterns for using the Odos SDK in production applications.

## Table of Contents

1. [Basic Swaps](#basic-swaps)
2. [Advanced Quote Handling](#advanced-quote-handling)
3. [Error Handling Patterns](#error-handling-patterns)
4. [Multi-Chain Operations](#multi-chain-operations)
5. [Testing Strategies](#testing-strategies)
6. [Production Patterns](#production-patterns)
7. [Integration Examples](#integration-examples)

## Basic Swaps

### Simple Token Swap

The most common use case - swapping one token for another:

```rust
use odos_sdk::prelude::*;
use alloy_primitives::{Address, U256};

async fn swap_usdc_for_weth(
    client: &OdosClient,
    usdc: Address,
    weth: Address,
    amount: U256,
    user: Address,
) -> Result<TransactionRequest> {
    client.swap()
        .chain(Chain::ethereum())
        .from_token(usdc, amount)
        .to_token(weth)
        .slippage(Slippage::percent(0.5)?)
        .signer(user)
        .build_transaction()
        .await
}
```

### Swap with Custom Recipient

Send the output tokens to a different address:

```rust
use odos_sdk::prelude::*;

async fn swap_to_recipient(
    client: &OdosClient,
    from_token: Address,
    to_token: Address,
    amount: U256,
    signer: Address,
    recipient: Address,
) -> Result<TransactionRequest> {
    client.swap()
        .chain(Chain::arbitrum())
        .from_token(from_token, amount)
        .to_token(to_token)
        .slippage(Slippage::bps(50)?)  // 0.5% in basis points
        .signer(signer)
        .recipient(recipient)  // Different recipient
        .build_transaction()
        .await
}
```

### Swap with Variable Slippage

Adjust slippage based on market conditions:

```rust
use odos_sdk::prelude::*;

async fn swap_with_dynamic_slippage(
    client: &OdosClient,
    token_in: Address,
    token_out: Address,
    amount: U256,
    user: Address,
    is_volatile: bool,
) -> Result<TransactionRequest> {
    // Higher slippage for volatile markets
    let slippage = if is_volatile {
        Slippage::percent(1.0)?  // 1% for volatile
    } else {
        Slippage::percent(0.5)?  // 0.5% for stable
    };

    client.swap()
        .chain(Chain::base())
        .from_token(token_in, amount)
        .to_token(token_out)
        .slippage(slippage)
        .signer(user)
        .build_transaction()
        .await
}
```

## Advanced Quote Handling

### Quote-First Workflow

Get a quote first to show users the expected output before executing:

```rust
use odos_sdk::{OdosClient, QuoteRequest, AssemblyRequest};
use alloy_primitives::{Address, U256};
use alloy_chains::NamedChain;

struct SwapQuote {
    input_amount: String,
    expected_output: String,
    gas_estimate: u64,
    price_impact: f64,
    path_id: String,
}

async fn get_swap_quote(
    client: &OdosClient,
    chain_id: u64,
    token_in: Address,
    token_out: Address,
    amount_in: U256,
    user: Address,
) -> Result<SwapQuote> {
    let quote_request = QuoteRequest::builder()
        .chain_id(chain_id)
        .input_tokens(vec![(token_in, amount_in).into()])
        .output_tokens(vec![(token_out, 1).into()])
        .slippage_limit_percent(0.5)
        .user_addr(user)
        .compact(false)
        .simple(false)
        .referral_code(0)
        .disable_rfqs(false)
        .build();

    let quote = client.quote(&quote_request).await?;

    Ok(SwapQuote {
        input_amount: quote.in_amount().unwrap_or(&"0".to_string()).clone(),
        expected_output: quote.out_amount().unwrap_or(&"0".to_string()).clone(),
        gas_estimate: quote.gas_estimate(),
        price_impact: quote.price_impact(),
        path_id: quote.path_id().to_string(),
    })
}

async fn execute_swap_from_quote(
    client: &OdosClient,
    chain: NamedChain,
    token_in: Address,
    amount_in: U256,
    signer: Address,
    recipient: Address,
    path_id: String,
) -> Result<TransactionRequest> {
    let assembly_request = AssemblyRequest::builder()
        .chain(chain)
        .router_address(chain.v2_router_address()?)
        .signer_address(signer)
        .output_recipient(recipient)
        .token_address(token_in)
        .token_amount(amount_in)
        .path_id(path_id)
        .build();

    client.assemble(&assembly_request).await
}
```

### Quote Comparison Across Chains

Compare quotes across multiple chains to find the best price:

```rust
use odos_sdk::{OdosClient, QuoteRequest, Chain};
use alloy_primitives::{Address, U256};
use futures::future::join_all;

#[derive(Debug)]
struct ChainQuote {
    chain: Chain,
    output_amount: String,
    gas_estimate: u64,
}

async fn compare_quotes_across_chains(
    client: &OdosClient,
    token_in: Address,
    token_out: Address,
    amount: U256,
    user: Address,
) -> Result<Vec<ChainQuote>> {
    let chains = vec![
        Chain::ethereum(),
        Chain::arbitrum(),
        Chain::optimism(),
        Chain::base(),
    ];

    // Request quotes in parallel
    let quote_futures = chains.iter().map(|chain| {
        let client = client.clone();
        let token_in = token_in;
        let token_out = token_out;
        let amount = amount;
        let user = user;
        let chain = *chain;

        async move {
            let request = QuoteRequest::builder()
                .chain_id(chain.id())
                .input_tokens(vec![(token_in, amount).into()])
                .output_tokens(vec![(token_out, 1).into()])
                .slippage_limit_percent(0.5)
                .user_addr(user)
                .compact(false)
                .simple(false)
                .referral_code(0)
                .disable_rfqs(false)
                .build();

            match client.quote(&request).await {
                Ok(quote) => Some(ChainQuote {
                    chain,
                    output_amount: quote.out_amount().unwrap_or(&"0".to_string()).clone(),
                    gas_estimate: quote.gas_estimate(),
                }),
                Err(_) => None,
            }
        }
    });

    let results = join_all(quote_futures).await;
    Ok(results.into_iter().flatten().collect())
}
```

### Quote with Timeout

Set a custom timeout for quote requests:

```rust
use odos_sdk::{OdosClient, ClientConfig, RetryConfig};
use std::time::Duration;

async fn get_quote_with_timeout() -> Result<()> {
    let config = ClientConfig {
        timeout: Duration::from_secs(5),  // 5 second timeout
        retry_config: RetryConfig::no_retries(),
        ..Default::default()
    };

    let client = OdosClient::with_config(config)?;

    // Quote request will timeout after 5 seconds
    let quote = client.quote(&quote_request).await?;

    Ok(())
}
```

## Error Handling Patterns

### Comprehensive Error Handler

Handle all error types appropriately:

```rust
use odos_sdk::{OdosError, OdosClient, QuoteRequest, Result};
use std::time::Duration;

async fn robust_quote_request(
    client: &OdosClient,
    request: QuoteRequest,
    max_attempts: u32,
) -> Result<SingleQuoteResponse> {
    let mut attempts = 0;

    loop {
        attempts += 1;

        match client.quote(&request).await {
            Ok(quote) => return Ok(quote),

            Err(e) if e.is_rate_limit() => {
                if attempts >= max_attempts {
                    return Err(e);
                }

                println!("Rate limited, attempt {}/{}", attempts, max_attempts);

                if let Some(retry_after) = e.retry_after() {
                    tokio::time::sleep(retry_after).await;
                } else {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }

            Err(e) if e.is_timeout() => {
                if attempts >= max_attempts {
                    return Err(e);
                }

                println!("Timeout, retrying... ({}/{})", attempts, max_attempts);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }

            Err(e) => {
                // Log error details
                eprintln!("Error: {}", e);

                if let Some(code) = e.error_code() {
                    eprintln!("Error code: {}", code);

                    if code.is_validation_error() {
                        eprintln!("Validation error - check request parameters");
                        return Err(e);  // Don't retry validation errors
                    }

                    if code.is_no_viable_path() {
                        eprintln!("No routing path - try different tokens or amount");
                        return Err(e);  // Don't retry path errors
                    }
                }

                if let Some(trace_id) = e.trace_id() {
                    eprintln!("Trace ID: {}", trace_id);
                }

                // Check if retryable
                if e.is_retryable() && attempts < max_attempts {
                    println!("Retrying... ({}/{})", attempts, max_attempts);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                } else {
                    return Err(e);
                }
            }
        }
    }
}
```

### Graceful Degradation

Fallback to alternative strategies when errors occur:

```rust
use odos_sdk::{OdosClient, QuoteRequest, Result};

async fn get_quote_with_fallback(
    client: &OdosClient,
    primary_request: QuoteRequest,
    fallback_request: QuoteRequest,
) -> Result<SingleQuoteResponse> {
    // Try primary request
    match client.quote(&primary_request).await {
        Ok(quote) => {
            println!("Primary quote succeeded");
            Ok(quote)
        }
        Err(e) => {
            println!("Primary quote failed: {}, trying fallback", e);

            // Try fallback request
            match client.quote(&fallback_request).await {
                Ok(quote) => {
                    println!("Fallback quote succeeded");
                    Ok(quote)
                }
                Err(fallback_err) => {
                    eprintln!("Both primary and fallback failed");
                    Err(fallback_err)
                }
            }
        }
    }
}
```

### Error Metrics Collection

Track errors for monitoring and alerting:

```rust
use odos_sdk::{OdosError, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct ErrorMetrics {
    errors_by_category: HashMap<String, u64>,
    errors_by_code: HashMap<String, u64>,
    total_errors: u64,
}

impl ErrorMetrics {
    fn record_error(&mut self, error: &OdosError) {
        self.total_errors += 1;

        let category = error.category();
        *self.errors_by_category.entry(category.to_string()).or_insert(0) += 1;

        if let Some(code) = error.error_code() {
            *self.errors_by_code.entry(code.to_string()).or_insert(0) += 1;
        }
    }

    fn report(&self) {
        println!("Error Metrics:");
        println!("  Total errors: {}", self.total_errors);
        println!("  By category:");
        for (category, count) in &self.errors_by_category {
            println!("    {}: {}", category, count);
        }
        println!("  By code:");
        for (code, count) in &self.errors_by_code {
            println!("    {}: {}", code, count);
        }
    }
}

// Usage
async fn monitored_quote_request(
    client: &OdosClient,
    request: QuoteRequest,
    metrics: Arc<Mutex<ErrorMetrics>>,
) -> Result<SingleQuoteResponse> {
    match client.quote(&request).await {
        Ok(quote) => Ok(quote),
        Err(e) => {
            metrics.lock().unwrap().record_error(&e);
            Err(e)
        }
    }
}
```

## Multi-Chain Operations

### Chain-Specific Router Selection

Use the optimal router for each chain:

```rust
use odos_sdk::{OdosChain, OdosRouterSelection};
use alloy_chains::NamedChain;
use alloy_primitives::Address;

fn get_optimal_router(chain: NamedChain) -> Result<Address> {
    // Use V3 if available, otherwise V2
    if chain.supports_v3() {
        chain.v3_router_address()
    } else {
        chain.v2_router_address()
    }
}

// Or use the built-in recommendation
fn get_recommended_router(chain: NamedChain) -> Result<Address> {
    chain.recommended_router_address()
}
```

### Check Chain Support

Verify chain support before operations:

```rust
use odos_sdk::{OdosChain, get_supported_chains};
use alloy_chains::NamedChain;

fn validate_chain(chain: NamedChain) -> Result<()> {
    if !chain.supports_odos() {
        return Err(OdosError::UnsupportedChain {
            chain_id: chain.id(),
        });
    }

    println!("Chain {} support:", chain.id());
    println!("  V2: {}", chain.supports_v2());
    println!("  V3: {}", chain.supports_v3());
    println!("  Limit Orders: {}", chain.supports_lo());

    Ok(())
}

fn list_all_supported_chains() {
    let chains = get_supported_chains();
    println!("Supported chains: {} total", chains.len());

    for &chain_id in chains {
        println!("  Chain ID {}", chain_id);
    }
}
```

## Testing Strategies

### Mock Client for Testing

Create a testable wrapper around the Odos client:

```rust
use odos_sdk::{OdosClient, QuoteRequest, SingleQuoteResponse, Result};
use async_trait::async_trait;

#[async_trait]
trait QuoteProvider {
    async fn get_quote(&self, request: &QuoteRequest) -> Result<SingleQuoteResponse>;
}

struct OdosQuoteProvider {
    client: OdosClient,
}

#[async_trait]
impl QuoteProvider for OdosQuoteProvider {
    async fn get_quote(&self, request: &QuoteRequest) -> Result<SingleQuoteResponse> {
        self.client.quote(request).await
    }
}

// Mock implementation for testing
struct MockQuoteProvider {
    responses: Vec<Result<SingleQuoteResponse>>,
    call_count: std::sync::Arc<std::sync::Mutex<usize>>,
}

#[async_trait]
impl QuoteProvider for MockQuoteProvider {
    async fn get_quote(&self, _request: &QuoteRequest) -> Result<SingleQuoteResponse> {
        let mut count = self.call_count.lock().unwrap();
        let index = *count;
        *count += 1;

        self.responses[index].clone()
    }
}

// Your business logic using the trait
async fn process_swap<P: QuoteProvider>(
    provider: &P,
    request: QuoteRequest,
) -> Result<()> {
    let quote = provider.get_quote(&request).await?;
    println!("Quote: {}", quote.path_id());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_swap() {
        let mock = MockQuoteProvider {
            responses: vec![Ok(/* mock quote */)],
            call_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
        };

        let request = /* build request */;
        let result = process_swap(&mock, request).await;

        assert!(result.is_ok());
    }
}
```

### Integration Tests

Test against real API with appropriate safeguards:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use odos_sdk::{OdosClient, QuoteRequest, Chain};

    #[tokio::test]
    #[ignore]  // Run with: cargo test -- --ignored
    async fn test_real_quote_request() {
        let client = OdosClient::new().unwrap();

        let request = QuoteRequest::builder()
            .chain_id(1)
            .input_tokens(vec![(usdc_address, small_amount).into()])
            .output_tokens(vec![(weth_address, 1).into()])
            .slippage_limit_percent(0.5)
            .user_addr(test_address)
            .compact(false)
            .simple(false)
            .referral_code(0)
            .disable_rfqs(false)
            .build();

        let result = client.quote(&request).await;

        match result {
            Ok(quote) => {
                assert!(!quote.path_id().is_empty());
                println!("Integration test passed: {}", quote.path_id());
            }
            Err(e) if e.is_rate_limit() => {
                println!("Rate limited - test skipped");
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }
}
```

## Production Patterns

### Singleton Client Pattern

Share a single client across your application:

```rust
use odos_sdk::{OdosClient, ClientConfig};
use std::sync::Arc;
use once_cell::sync::OnceCell;

static ODOS_CLIENT: OnceCell<Arc<OdosClient>> = OnceCell::new();

fn get_odos_client() -> Arc<OdosClient> {
    ODOS_CLIENT
        .get_or_init(|| {
            let config = ClientConfig {
                // Production config
                ..Default::default()
            };

            Arc::new(OdosClient::with_config(config).expect("Failed to create Odos client"))
        })
        .clone()
}

// Usage in your application
async fn handler() {
    let client = get_odos_client();
    // Use client...
}
```

### Connection Pooling

Configure connection pooling for high-throughput applications:

```rust
use odos_sdk::{OdosClient, ClientConfig};
use std::time::Duration;

fn create_production_client() -> Result<OdosClient> {
    let config = ClientConfig {
        max_connections: 50,  // Higher connection pool
        pool_idle_timeout: Duration::from_secs(90),
        timeout: Duration::from_secs(30),
        connect_timeout: Duration::from_secs(10),
        ..Default::default()
    };

    OdosClient::with_config(config)
}
```

### Graceful Shutdown

Handle shutdown gracefully:

```rust
use odos_sdk::OdosClient;
use tokio::sync::broadcast;

async fn run_with_graceful_shutdown(
    client: Arc<OdosClient>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                println!("Shutdown signal received");
                break;
            }
            result = process_swap(&client) => {
                match result {
                    Ok(_) => println!("Swap processed"),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
    }

    println!("Graceful shutdown complete");
}
```

## Integration Examples

### Web Server Integration (Axum)

Integrate with an Axum web server:

```rust
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use odos_sdk::{OdosClient, QuoteRequest};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

struct AppState {
    odos_client: OdosClient,
}

#[derive(Deserialize)]
struct QuoteRequestDto {
    chain_id: u64,
    token_in: String,
    token_out: String,
    amount_in: String,
    user_address: String,
}

#[derive(Serialize)]
struct QuoteResponseDto {
    path_id: String,
    input_amount: String,
    output_amount: String,
    gas_estimate: u64,
}

async fn get_quote_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QuoteRequestDto>,
) -> Result<Json<QuoteResponseDto>, StatusCode> {
    let token_in = payload.token_in.parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let token_out = payload.token_out.parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let amount = payload.amount_in.parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let user = payload.user_address.parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let request = QuoteRequest::builder()
        .chain_id(payload.chain_id)
        .input_tokens(vec![(token_in, amount).into()])
        .output_tokens(vec![(token_out, 1).into()])
        .slippage_limit_percent(0.5)
        .user_addr(user)
        .compact(false)
        .simple(false)
        .referral_code(0)
        .disable_rfqs(false)
        .build();

    let quote = state.odos_client.quote(&request).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(QuoteResponseDto {
        path_id: quote.path_id().to_string(),
        input_amount: quote.in_amount().unwrap_or(&"0".to_string()).clone(),
        output_amount: quote.out_amount().unwrap_or(&"0".to_string()).clone(),
        gas_estimate: quote.gas_estimate(),
    }))
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        odos_client: OdosClient::new().expect("Failed to create client"),
    });

    let app = Router::new()
        .route("/quote", post(get_quote_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

### CLI Tool

Build a command-line tool:

```rust
use clap::Parser;
use odos_sdk::prelude::*;

#[derive(Parser)]
#[command(name = "odos-swap")]
#[command(about = "Get quotes from Odos protocol")]
struct Cli {
    #[arg(long)]
    chain_id: u64,

    #[arg(long)]
    token_in: String,

    #[arg(long)]
    token_out: String,

    #[arg(long)]
    amount: String,

    #[arg(long)]
    user: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let client = OdosClient::new()?;

    let token_in = cli.token_in.parse()?;
    let token_out = cli.token_out.parse()?;
    let amount = cli.amount.parse()?;
    let user = cli.user.parse()?;

    let request = QuoteRequest::builder()
        .chain_id(cli.chain_id)
        .input_tokens(vec![(token_in, amount).into()])
        .output_tokens(vec![(token_out, 1).into()])
        .slippage_limit_percent(0.5)
        .user_addr(user)
        .compact(false)
        .simple(false)
        .referral_code(0)
        .disable_rfqs(false)
        .build();

    match client.quote(&request).await {
        Ok(quote) => {
            println!("✓ Quote received");
            println!("  Path ID: {}", quote.path_id());
            println!("  Input: {} tokens", quote.in_amount().unwrap_or(&"0".to_string()));
            println!("  Output: {} tokens", quote.out_amount().unwrap_or(&"0".to_string()));
            println!("  Gas: {}", quote.gas_estimate());
        }
        Err(e) => {
            eprintln!("✗ Error: {}", e);
            if let Some(trace_id) = e.trace_id() {
                eprintln!("  Trace ID: {}", trace_id);
            }
            std::process::exit(1);
        }
    }

    Ok(())
}
```

## Best Practices Summary

1. **Client Management**
   - Create one client instance and share it
   - Configure appropriate timeouts and connection pools
   - Use conservative retry configs for production

2. **Error Handling**
   - Always check error codes for specific conditions
   - Handle rate limits gracefully with backoff
   - Log trace IDs for debugging

3. **Performance**
   - Reuse client connections
   - Consider quote caching for repeated requests
   - Use parallel requests when appropriate

4. **Security**
   - Validate all input addresses
   - Use appropriate slippage limits
   - Never expose private keys in logs

5. **Testing**
   - Use traits for testability
   - Mock external dependencies
   - Test error paths thoroughly

## More Resources

- [Getting Started Guide](GETTING_STARTED.md)
- [API Documentation](https://docs.rs/odos-sdk)
- [Odos Protocol Docs](https://docs.odos.xyz)
- [GitHub Repository](https://github.com/semiotic-ai/odos-sdk)
