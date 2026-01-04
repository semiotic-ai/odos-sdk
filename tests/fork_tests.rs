// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

//! Fork-based integration tests using Anvil.
//!
//! These tests verify router contract interactions against a forked mainnet state.
//! They require Anvil to be installed and available in PATH.
//!
//! Run with: `cargo test --test fork_tests -- --ignored`

use alloy_chains::NamedChain;
use alloy_network::Ethereum;
use alloy_primitives::Address;
use alloy_provider::ProviderBuilder;
use odos_sdk::{OdosChain, V3Router, ODOS_V3};

/// Fork block number for reproducible tests
const FORK_BLOCK: u64 = 21_000_000;

/// Tests that we can read the V3 router owner on a forked mainnet.
#[tokio::test]
#[ignore = "requires Anvil and network access"]
async fn test_v3_router_owner_on_fork() {
    let provider = ProviderBuilder::new().connect_anvil_with_config(|anvil| {
        anvil
            .fork("https://eth.llamarpc.com")
            .fork_block_number(FORK_BLOCK)
    });

    let router: V3Router<Ethereum, _> = V3Router::new(ODOS_V3, provider);

    let owner = router.owner().await.expect("should read owner");

    // The V3 router should have a valid owner address
    assert_ne!(owner, Address::ZERO, "owner should not be zero address");
}

/// Tests that we can read the V3 router liquidator address on a forked mainnet.
#[tokio::test]
#[ignore = "requires Anvil and network access"]
async fn test_v3_router_liquidator_on_fork() {
    let provider = ProviderBuilder::new().connect_anvil_with_config(|anvil| {
        anvil
            .fork("https://eth.llamarpc.com")
            .fork_block_number(FORK_BLOCK)
    });

    let router: V3Router<Ethereum, _> = V3Router::new(ODOS_V3, provider);

    let liquidator = router
        .liquidator_address()
        .await
        .expect("should read liquidator");

    // The liquidator address should be set
    assert_ne!(
        liquidator,
        Address::ZERO,
        "liquidator should not be zero address"
    );
}

/// Tests that the V2 router addresses are valid on forked mainnet.
#[tokio::test]
#[ignore = "requires Anvil and network access"]
async fn test_v2_router_exists_on_fork() {
    use odos_sdk::V2Router;

    let provider = ProviderBuilder::new().connect_anvil_with_config(|anvil| {
        anvil
            .fork("https://eth.llamarpc.com")
            .fork_block_number(FORK_BLOCK)
    });

    let v2_address = NamedChain::Mainnet
        .v2_router_address()
        .expect("mainnet should have V2 router");

    let router: V2Router<Ethereum, _> = V2Router::new(v2_address, provider);

    let owner = router.owner().await.expect("should read owner");
    assert_ne!(owner, Address::ZERO, "owner should not be zero address");
}

/// Tests chain support detection.
#[test]
fn test_chain_support() {
    // All major chains should support V3
    assert!(NamedChain::Mainnet.supports_v3());
    assert!(NamedChain::Arbitrum.supports_v3());
    assert!(NamedChain::Optimism.supports_v3());
    assert!(NamedChain::Base.supports_v3());
    assert!(NamedChain::Polygon.supports_v3());

    // All major chains should support V2
    assert!(NamedChain::Mainnet.supports_v2());
    assert!(NamedChain::Arbitrum.supports_v2());
    assert!(NamedChain::Optimism.supports_v2());
    assert!(NamedChain::Base.supports_v2());
    assert!(NamedChain::Polygon.supports_v2());
}

/// Tests that the V3 router address is the same across all chains (CREATE2 deployment).
#[test]
fn test_v3_unified_address() {
    let chains = [
        NamedChain::Mainnet,
        NamedChain::Arbitrum,
        NamedChain::Optimism,
        NamedChain::Base,
        NamedChain::Polygon,
        NamedChain::BinanceSmartChain,
        NamedChain::Avalanche,
    ];

    for chain in chains {
        let v3_address: Address = chain.v3_router_address().expect("should have V3 address");
        assert_eq!(
            v3_address, ODOS_V3,
            "V3 router should have unified address on {chain:?}"
        );
    }
}
