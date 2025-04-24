# Odos Rust SDK

This is a [Rust](https://www.rust-lang.org/) SDK for [Odos](https://docs.odos.xyz/).

## One-to-one Swap Example

```rust
use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};
use odos_sdk::{Erc20, OdosSorV2, QuoteRequest, Swap};

/// Token address of the token to swap
const TOKEN: &str = "0x4200000000000000000000000000000000000006";
/// Top holder of the token at time of writing according to https://etherscan.io/token/0x4200000000000000000000000000000000000006#balances
const HOLDER: &str = "0xb2cc224c1c9feE385f8ad6a55b4d94E92359DC59";
/// Odos v2 router address on Base
const ROUTER: &str = "0x19cEeAd7105607Cd444F5ad10dd51356436095a1";
/// Recipient of the swap
const RECIPIENT: &str = "0x83384D138420436f4b0DaE97b02002dd5011a7D9";

const USDC: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";

let chain = NamedChain::Base;

let token_address = TOKEN.parse::<Address>().unwrap();

let token_contract = Erc20::new(token_address, &self.provider);
let balance = token_contract.balance_of(account).await?;

let quote_request = QuoteRequest::builder()
    .chain_id(chain.into())
    .input_tokens(vec![(token_address, balance).into()])
    .output_tokens(vec![(USDC.parse::<Address>().unwrap(), 1).into()])
    .slippage_limit_percent(1.0)
    .user_addr(signer_provider.signer_address().to_string())
    .compact(false)
    .simple(false)
    .referral_code(0)
    .disable_rfqs(true)
    .build();

let sor_client = OdosSorV2::new();

let quote = sor_client.get_swap_quote(&quote_request).await.unwrap();

let token_contract = Erc20::new(token_address, root_provider.clone());

// Approve the router to spend the token on behalf of the signer
let pending_tx = token_contract
    .approve(
        signer_provider.signer_address(),
        ROUTER.parse::<Address>().unwrap(),
        token_balance,
    )
    .await
    .unwrap();

let receipt = pending_tx.get_receipt().await.unwrap();

// Assert the transaction was successful
assert!(receipt.status());

let swap_params = Swap::builder()
    .chain(chain)
    .router_address(ROUTER.parse::<Address>().unwrap())
    .signer_address(signer_provider.signer_address())
    .output_recipient(RECIPIENT.parse::<Address>().unwrap())
    .token_address(token_address)
    .token_amount(token_balance)
    .path_id(String::from(quote.path_id()))
    .build();

let tx_request = sor_client
    .build_base_transaction(&swap_params)
    .await
    .unwrap();

// User can apply custom gas parameters to the transaction as they see fit

let pending_tx = signer_provider.send_transaction(tx_request).await.unwrap();

let receipt = pending_tx.get_receipt().await.unwrap();

// Assert the transaction was successful
assert!(receipt.status());
```
