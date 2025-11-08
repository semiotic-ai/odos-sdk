//! # Odos Protocol Contract Addresses
//!
//! This module contains the verified contract addresses for the Odos protocol across
//! multiple blockchain networks. Odos is a decentralized exchange aggregator that
//! provides optimal routing for token swaps through their Smart Order Routing (SOR) system.
//!
//! ## Contract Types
//!
//! - **Limit Order V2 (LO)**: Specialized contract for limit order functionality (chain-specific addresses)
//! - **V2 Router**: The main swap router contract supporting single and multi-token swaps,
//!   permit2 integration, and referral systems. Each chain has its own deployed instance.
//! - **V3 Router**: Next-generation router with enhanced features (unified address across chains)
//!
//! ## Usage
//!
//! **Recommended: Type-safe trait-based approach**
//! ```rust
//! use odos_sdk::{OdosChain, OdosRouterSelection};
//! use alloy_chains::NamedChain;
//!
//! // Type-safe router address lookup
//! let v2_router = NamedChain::Mainnet.v2_router_address()?;
//! let v3_router = NamedChain::Mainnet.v3_router_address()?;
//!
//! // Get both addresses at once
//! let (v2, v3) = NamedChain::Arbitrum.both_router_addresses()?;
//!
//! // Safe lookups that don't panic
//! if let Some(router_addr) = NamedChain::Mainnet.try_v3_router_address() {
//!     // Use the router address with your provider
//!     println!("V3 router address: {router_addr}");
//! }
//!
//! // Smart router selection
//! let recommended = NamedChain::Base.recommended_router_address()?;
//! # Ok::<(), odos_sdk::OdosChainError>(())
//! ```
//!
//! **Alternative: Chain ID-based approach**
//! ```rust
//! use odos_sdk::{get_v2_router_by_chain_id, get_v3_router_by_chain_id};
//!
//! // For cases where you only have chain IDs
//! if let Some(router_addr) = get_v3_router_by_chain_id(1) {
//!     println!("Ethereum V3 router: {router_addr}");
//! }
//! ```
//!
//! ## Security Considerations
//!
//! ⚠️ **CRITICAL**: Always verify contract addresses against official sources before use.
//! These addresses are immutable and have been verified against the official Odos deployments,
//! but you should:
//!
//! - Cross-reference with official Odos documentation
//! - Verify the contract code matches expected interfaces
//! - Use appropriate slippage protection in swaps
//! - Consider gas costs and execution deadlines
//!
//! ## Chain Support
//!
//! Router availability varies by chain:
//!
//! ### All Routers (LO, V2, V3)
//! - **Layer 1**: Ethereum
//! - **Layer 2**: Arbitrum, Optimism, Polygon, Base, Scroll, Linea, zkSync, Mantle, Mode
//! - **Sidechains**: BSC, Avalanche, Fraxtal, Sonic, Unichain
//!
//! ### LO + V3 Only (no V2)
//! - **Layer 2**: Berachain
//!
//! ## Router Type Differences
//!
//! - **LO (Limit Order V2)**: Limit order functionality, available on all chains (chain-specific addresses)
//! - **V2**: Chain-specific deployments, mature and battle-tested, available on all chains except Berachain
//! - **V3**: Unified address across all chains, enhanced features, production-ready, available on all chains

use alloy_chains::NamedChain;
use alloy_primitives::{address, Address};

use crate::OdosChain;

// =============================================================================
// V2 Router Addresses (Chain-Specific Deployments)
// =============================================================================

/// **Arbitrum One** - V2 Router contract address
///
/// Chain ID: 42161
///
/// **Verified on**: <https://arbiscan.io/address/0xa669e7a0d4b3e4fa48af2de86bd4cd7126be4e13>
pub const ODOS_V2_ARBITRUM_ROUTER: Address = address!("a669e7a0d4b3e4fa48af2de86bd4cd7126be4e13");

/// **Base** - V2 Router contract address
///
/// Chain ID: 8453
///
/// **Verified on**: <https://basescan.org/address/0x19cEeAd7105607Cd444F5ad10dd51356436095a1>
pub const ODOS_V2_BASE_ROUTER: Address = address!("19cEeAd7105607Cd444F5ad10dd51356436095a1");

/// **BNB Smart Chain** - V2 Router contract address
///
/// Chain ID: 56
///
/// **Verified on**: <https://bscscan.com/address/0x89b8aa89fdd0507a99d334cbe3c808fafc7d850e>
pub const ODOS_V2_BSC_ROUTER: Address = address!("89b8AA89FDd0507a99d334CBe3C808fAFC7d850E");

/// **Ethereum Mainnet** - V2 Router contract address
///
/// Chain ID: 1
///
/// **Verified on**: <https://etherscan.io/address/0xcf5540fffcdc3d510b18bfca6d2b9987b0772559>
pub const ODOS_V2_ETHEREUM_ROUTER: Address = address!("Cf5540fFFCdC3d510B18bFcA6d2b9987b0772559");

/// **Optimism** - V2 Router contract address
///
/// Chain ID: 10
///
/// **Verified on**: <https://optimistic.etherscan.io/address/0xca423977156bb05b13a2ba3b76bc5419e2fe9680>
pub const ODOS_V2_OP_ROUTER: Address = address!("Ca423977156BB05b13A2BA3b76Bc5419E2fE9680");

/// **Avalanche C-Chain** - V2 Router contract address
///
/// Chain ID: 43114
///
/// **Verified on**: <https://snowtrace.io/address/0x88de50b233052e4fb783d4f6db78cc34fea3e9fc>
pub const ODOS_V2_AVALANCHE_ROUTER: Address = address!("88de50B233052e4Fb783d4F6db78Cc34fEa3e9FC");

/// **Polygon** - V2 Router contract address
///
/// Chain ID: 137
///
/// **Verified on**: <https://polygonscan.com/address/0x4e3288c9ca110bcc82bf38f09a7b425c095d92bf>
pub const ODOS_V2_POLYGON_ROUTER: Address = address!("4e3288c9ca110bcc82bf38f09a7b425c095d92bf");

/// **Fraxtal** - V2 Router contract address
///
/// Chain ID: 252
///
/// **Verified on**: <https://fraxscan.com/address/0x56c85a254DD12eE8D9C04049a4ab62769Ce98210>
pub const ODOS_V2_FRAXTAL_ROUTER: Address = address!("56c85a254DD12eE8D9C04049a4ab62769Ce98210");

/// **Linea** - V2 Router contract address
///
/// Chain ID: 59144
///
/// **Verified on**: <https://linea.blockscout.com/address/0x2d8879046f1559E53eb052E949e9544bCB72f414>
pub const ODOS_V2_LINEA_ROUTER: Address = address!("2d8879046f1559E53eb052E949e9544bCB72f414");

/// **Mantle** - V2 Router contract address
///
/// Chain ID: 5000
///
/// **Verified on**: <https://mantlescan.xyz/address/0xD9F4e85489aDCD0bAF0Cd63b4231c6af58c26745>
pub const ODOS_V2_MANTLE_ROUTER: Address = address!("D9F4e85489aDCD0bAF0Cd63b4231c6af58c26745");

/// **Mode** - V2 Router contract address
///
/// Chain ID: 34443
///
/// **Verified on**: <https://explorer.mode.network/address/0x7E15EB462cdc67Cf92Af1f7102465a8F8c784874>
pub const ODOS_V2_MODE_ROUTER: Address = address!("7E15EB462cdc67Cf92Af1f7102465a8F8c784874");

/// **Scroll** - V2 Router contract address
///
/// Chain ID: 534352
///
/// **Verified on**: <https://explorer.scroll.io/address/0xbFe03C9E20a9Fc0b37de01A172F207004935E0b1>
pub const ODOS_V2_SCROLL_ROUTER: Address = address!("bFe03C9E20a9Fc0b37de01A172F207004935E0b1");

/// **Sonic** - V2 Router contract address
///
/// Chain ID: 146
///
/// **Verified on**: <https://sonar.explorer.sonar.watch/address/0xaC041Df48dF9791B0654f1Dbbf2CC8450C5f2e9D>
pub const ODOS_V2_SONIC_ROUTER: Address = address!("aC041Df48dF9791B0654f1Dbbf2CC8450C5f2e9D");

/// **zkSync Era** - V2 Router contract address
///
/// Chain ID: 324
///
/// **Verified on**: <https://explorer.zksync.io/address/0x4bBa932E9792A2b917D47830C93a9BC79320E4f7>
pub const ODOS_V2_ZKSYNC_ROUTER: Address = address!("4bBa932E9792A2b917D47830C93a9BC79320E4f7");

/// **Unichain** - V2 Router contract address
///
/// Chain ID: 1301
///
/// **Verified on**: <https://uniscan.xyz/address/0x6409722f3a1c4486a3b1fe566cbdd5e9d946a1f3>
pub const ODOS_V2_UNICHAIN_ROUTER: Address = address!("6409722F3a1C4486A3b1FE566cBDd5e9D946A1f3");

// =============================================================================
// Limit Order V2 Router Addresses (Chain-Specific Deployments)
// =============================================================================

/// **Ethereum Mainnet** - Limit Order V2 Router contract address
///
/// Chain ID: 1
///
/// **Verified on**: <https://etherscan.io/address/0x5F79636fa7bc622eA48802E6cf80A5dae814daE1>
pub const ODOS_LO_ETHEREUM_ROUTER: Address = address!("5F79636fa7bc622eA48802E6cf80A5dae814daE1");

/// **Optimism** - Limit Order V2 Router contract address
///
/// Chain ID: 10
///
/// **Verified on**: <https://optimistic.etherscan.io/address/0xcbF3822A63B7867cD602317fB4aE3ca864826ef8>
pub const ODOS_LO_OP_ROUTER: Address = address!("cbF3822A63B7867cD602317fB4aE3ca864826ef8");

/// **BNB Smart Chain** - Limit Order V2 Router contract address
///
/// Chain ID: 56
///
/// **Verified on**: <https://bscscan.com/address/0x0D4aB12E62D17f037D43F018Da18FF623e1AF3B2>
pub const ODOS_LO_BSC_ROUTER: Address = address!("0D4aB12E62D17f037D43F018Da18FF623e1AF3B2");

/// **Polygon** - Limit Order V2 Router contract address
///
/// Chain ID: 137
///
/// **Verified on**: <https://polygonscan.com/address/0x93052961c75c92Fd5d6362655936C239EF2D5336>
pub const ODOS_LO_POLYGON_ROUTER: Address = address!("93052961c75c92Fd5d6362655936C239EF2D5336");

/// **Berachain** - Limit Order V2 Router contract address
///
/// Chain ID: 80084
///
/// **Verified on**: Berachain explorer
pub const ODOS_LO_BERACHAIN_ROUTER: Address = address!("3236d3d12b7981aa5043bd66d1b9d6856bf764dc");

/// **Arbitrum One** - Limit Order V2 Router contract address
///
/// Chain ID: 42161
///
/// **Verified on**: <https://arbiscan.io/address/0x7432657cDda02226ac2aAc9d8f552Ee9613B064e>
pub const ODOS_LO_ARBITRUM_ROUTER: Address = address!("7432657cDda02226ac2aAc9d8f552Ee9613B064e");

/// **Avalanche C-Chain** - Limit Order V2 Router contract address
///
/// Chain ID: 43114
///
/// **Verified on**: <https://snowtrace.io/address/0xcc0126349d1bD892D1C53381E68dBF0c8F0E045e>
pub const ODOS_LO_AVALANCHE_ROUTER: Address = address!("cc0126349d1bD892D1C53381E68dBF0c8F0E045e");

/// **Base** - Limit Order V2 Router contract address
///
/// Chain ID: 8453
///
/// **Verified on**: <https://basescan.org/address/0xeDeAfdEf0901eF74Ee28c207BE8424D3B353D97A>
pub const ODOS_LO_BASE_ROUTER: Address = address!("eDeAfdEf0901eF74Ee28c207BE8424D3B353D97A");

/// **Fraxtal** - Limit Order V2 Router contract address
///
/// Chain ID: 252
///
/// **Verified on**: <https://fraxscan.com/address/0x5E0aFaD0f658f9689806296e0509AfFC191d9a09>
pub const ODOS_LO_FRAXTAL_ROUTER: Address = address!("5E0aFaD0f658f9689806296e0509AfFC191d9a09");

/// **Linea** - Limit Order V2 Router contract address
///
/// Chain ID: 59144
///
/// **Verified on**: <https://linea.blockscout.com/address/0xb3a9B56056a5c93F468dF62579b9A5BEa1741069>
pub const ODOS_LO_LINEA_ROUTER: Address = address!("b3a9B56056a5c93F468dF62579b9A5BEa1741069");

/// **Mantle** - Limit Order V2 Router contract address
///
/// Chain ID: 5000
///
/// **Verified on**: <https://mantlescan.xyz/address/0xa05A88037402d869b7CA69F5bEc098E19BeDaFbB>
pub const ODOS_LO_MANTLE_ROUTER: Address = address!("a05A88037402d869b7CA69F5bEc098E19BeDaFbB");

/// **Mode** - Limit Order V2 Router contract address
///
/// Chain ID: 34443
///
/// **Verified on**: <https://explorer.mode.network/address/0x8073e286DaDc6d92BefC8f436c5BcDFcE213e681>
pub const ODOS_LO_MODE_ROUTER: Address = address!("8073e286DaDc6d92BefC8f436c5BcDFcE213e681");

/// **Scroll** - Limit Order V2 Router contract address
///
/// Chain ID: 534352
///
/// **Verified on**: <https://explorer.scroll.io/address/0x468633515c46EfFCC77Caa949ce8775505e5deDA>
pub const ODOS_LO_SCROLL_ROUTER: Address = address!("468633515c46EfFCC77Caa949ce8775505e5deDA");

/// **Sonic** - Limit Order V2 Router contract address
///
/// Chain ID: 146
///
/// **Verified on**: Sonic explorer
pub const ODOS_LO_SONIC_ROUTER: Address = address!("B9CBD870916e9Ffc52076Caa714f85a022B7f330");

/// **zkSync Era** - Limit Order V2 Router contract address
///
/// Chain ID: 324
///
/// **Verified on**: <https://explorer.zksync.io/address/0x74ab8c1247aE3C5FFFD9F85781F31751bdd98E73>
pub const ODOS_LO_ZKSYNC_ROUTER: Address = address!("74ab8c1247aE3C5FFFD9F85781F31751bdd98E73");

/// **Unichain** - Limit Order V2 Router contract address
///
/// Chain ID: 1301
///
/// **Verified on**: <https://uniscan.xyz/address/0x372d96eDA72bEA64dfCa3577d04382E4dbE2Ff2b>
pub const ODOS_LO_UNICHAIN_ROUTER: Address = address!("372d96eDA72bEA64dfCa3577d04382E4dbE2Ff2b");

// =============================================================================
// V3 Router Address (Unified Across All Chains)
// =============================================================================

/// **Odos V3 Router** - Next-generation router contract
///
/// Unlike V2, the V3 router uses the same address across all supported chains,
/// following the CREATE2 deterministic deployment pattern.
///
/// **Features**:
/// - Unified address across all chains
/// - Enhanced swap routing algorithms
/// - Improved gas efficiency
/// - Advanced MEV protection
///
/// **Example verification**: <https://snowscan.xyz/address/0x0D05a7D3448512B78fa8A9e46c4872C88C4a0D05>
pub const ODOS_V3: Address = address!("0D05a7D3448512B78fa8A9e46c4872C88C4a0D05");

// =============================================================================
// Utility Functions (Built on top of the OdosChain trait)
// =============================================================================

/// Get the V2 router address for a specific chain ID
///
/// This function leverages the `OdosChain` trait to provide chain ID-based
/// lookups while maintaining a single source of truth for chain support.
///
/// # Arguments
///
/// * `chain_id` - The chain ID to look up
///
/// # Returns
///
/// * `Some(address)` - The router address if supported
/// * `None` - If the chain is not supported
///
/// # Example
///
/// ```rust
/// use odos_sdk::get_v2_router_by_chain_id;
///
/// let ethereum_router = get_v2_router_by_chain_id(1);
/// assert!(ethereum_router.is_some());
///
/// let unsupported_chain = get_v2_router_by_chain_id(999999);
/// assert!(unsupported_chain.is_none());
/// ```
pub fn get_v2_router_by_chain_id(chain_id: u64) -> Option<Address> {
    let named_chain = NamedChain::try_from(chain_id).ok()?;

    // Check if the chain specifically supports V2 (not just Odos in general)
    if !named_chain.supports_v2() {
        return None;
    }

    // Return the V2 router address constant
    Some(match named_chain {
        NamedChain::Mainnet => ODOS_V2_ETHEREUM_ROUTER,
        NamedChain::Arbitrum => ODOS_V2_ARBITRUM_ROUTER,
        NamedChain::Optimism => ODOS_V2_OP_ROUTER,
        NamedChain::BinanceSmartChain => ODOS_V2_BSC_ROUTER,
        NamedChain::Polygon => ODOS_V2_POLYGON_ROUTER,
        NamedChain::Fraxtal => ODOS_V2_FRAXTAL_ROUTER,
        NamedChain::ZkSync => ODOS_V2_ZKSYNC_ROUTER,
        NamedChain::Unichain => ODOS_V2_UNICHAIN_ROUTER,
        NamedChain::Mantle => ODOS_V2_MANTLE_ROUTER,
        NamedChain::Base => ODOS_V2_BASE_ROUTER,
        NamedChain::Mode => ODOS_V2_MODE_ROUTER,
        NamedChain::Avalanche => ODOS_V2_AVALANCHE_ROUTER,
        NamedChain::Linea => ODOS_V2_LINEA_ROUTER,
        NamedChain::Scroll => ODOS_V2_SCROLL_ROUTER,
        NamedChain::Sonic => ODOS_V2_SONIC_ROUTER,
        _ => return None,
    })
}

/// Get the Limit Order V2 router address for a specific chain ID
///
/// This function leverages the `OdosChain` trait to provide chain ID-based
/// lookups while maintaining a single source of truth for chain support.
///
/// # Arguments
///
/// * `chain_id` - The chain ID to look up
///
/// # Returns
///
/// * `Some(address)` - The LO router address if supported
/// * `None` - If the chain doesn't have a Limit Order router
///
/// # Example
///
/// ```rust
/// use odos_sdk::get_lo_router_by_chain_id;
///
/// let ethereum_lo = get_lo_router_by_chain_id(1);
/// assert!(ethereum_lo.is_some());
///
/// let unsupported_chain = get_lo_router_by_chain_id(999999);
/// assert!(unsupported_chain.is_none());
/// ```
pub fn get_lo_router_by_chain_id(chain_id: u64) -> Option<Address> {
    let named_chain = NamedChain::try_from(chain_id).ok()?;

    // Check if the chain specifically supports LO
    if !named_chain.supports_lo() {
        return None;
    }

    // Return the LO router address constant
    Some(match named_chain {
        NamedChain::Mainnet => ODOS_LO_ETHEREUM_ROUTER,
        NamedChain::Optimism => ODOS_LO_OP_ROUTER,
        NamedChain::BinanceSmartChain => ODOS_LO_BSC_ROUTER,
        NamedChain::Polygon => ODOS_LO_POLYGON_ROUTER,
        NamedChain::Berachain => ODOS_LO_BERACHAIN_ROUTER,
        NamedChain::Arbitrum => ODOS_LO_ARBITRUM_ROUTER,
        NamedChain::Avalanche => ODOS_LO_AVALANCHE_ROUTER,
        NamedChain::Base => ODOS_LO_BASE_ROUTER,
        NamedChain::Fraxtal => ODOS_LO_FRAXTAL_ROUTER,
        NamedChain::Linea => ODOS_LO_LINEA_ROUTER,
        NamedChain::Mantle => ODOS_LO_MANTLE_ROUTER,
        NamedChain::Mode => ODOS_LO_MODE_ROUTER,
        NamedChain::Scroll => ODOS_LO_SCROLL_ROUTER,
        NamedChain::Sonic => ODOS_LO_SONIC_ROUTER,
        NamedChain::ZkSync => ODOS_LO_ZKSYNC_ROUTER,
        NamedChain::Unichain => ODOS_LO_UNICHAIN_ROUTER,
        _ => return None,
    })
}

/// Get the V3 router address for a specific chain ID
///
/// This function leverages the `OdosChain` trait to provide chain ID-based
/// lookups while maintaining a single source of truth for chain support.
///
/// # Arguments
///
/// * `chain_id` - The chain ID to check support for
///
/// # Returns
///
/// * `Some(address)` - The V3 router address if the chain is supported
/// * `None` - If V3 is not deployed on that chain
///
/// # Example
///
/// ```rust
/// use odos_sdk::get_v3_router_by_chain_id;
///
/// // Check if V3 is available on Ethereum
/// if let Some(v3_address) = get_v3_router_by_chain_id(1) {
///     println!("V3 available on Ethereum: {}", v3_address);
/// }
/// ```
pub fn get_v3_router_by_chain_id(chain_id: u64) -> Option<Address> {
    let named_chain = NamedChain::try_from(chain_id).ok()?;

    // Check if the chain specifically supports V3
    if !named_chain.supports_v3() {
        return None;
    }

    Some(ODOS_V3)
}

/// Get all supported chain IDs
///
/// This function queries the trait implementation to determine which
/// chains are supported. A chain is considered supported if it has at least
/// one router type (LO, V2, or V3) deployed.
///
/// # Returns
///
/// A vector of all supported chain IDs
///
/// # Example
///
/// ```rust
/// use odos_sdk::get_supported_chains;
///
/// let chains = get_supported_chains();
/// assert!(chains.contains(&1)); // Ethereum (has LO, V2, V3)
/// assert!(chains.contains(&42161)); // Arbitrum (has LO, V2, V3)
/// assert!(chains.contains(&80094)); // Berachain (has LO, V3 only)
/// ```
pub fn get_supported_chains() -> Vec<u64> {
    use NamedChain::*;

    let all_chains = [
        Mainnet,
        Optimism,
        BinanceSmartChain,
        Polygon,
        Fraxtal,
        ZkSync,
        Unichain,
        Mantle,
        Base,
        Mode,
        Arbitrum,
        Avalanche,
        Linea,
        Scroll,
        Sonic,
        Berachain,
    ];

    all_chains
        .iter()
        .filter_map(|&chain| {
            // A chain is supported if it has at least one router type
            if chain.supports_odos() {
                Some(chain as u64)
            } else {
                None
            }
        })
        .collect()
}

/// Get all chain IDs that support Limit Order V2 routers
///
/// # Returns
///
/// A vector of chain IDs that have LO router deployments
///
/// # Example
///
/// ```rust
/// use odos_sdk::get_supported_lo_chains;
///
/// let lo_chains = get_supported_lo_chains();
/// assert!(lo_chains.contains(&1)); // Ethereum
/// assert!(lo_chains.contains(&80094)); // Berachain
/// ```
pub fn get_supported_lo_chains() -> Vec<u64> {
    use NamedChain::*;

    let all_chains = [
        Mainnet,
        Optimism,
        BinanceSmartChain,
        Polygon,
        Fraxtal,
        ZkSync,
        Unichain,
        Mantle,
        Base,
        Mode,
        Arbitrum,
        Avalanche,
        Linea,
        Scroll,
        Sonic,
        Berachain,
    ];

    all_chains
        .iter()
        .filter_map(|&chain| {
            // Check if the chain supports LO using trait method
            if chain.supports_lo() {
                Some(chain as u64)
            } else {
                None
            }
        })
        .collect()
}

/// Get all chain IDs that support V2 routers
///
/// Note: Berachain does NOT support V2, only LO and V3
///
/// # Returns
///
/// A vector of chain IDs that have V2 router deployments
///
/// # Example
///
/// ```rust
/// use odos_sdk::get_supported_v2_chains;
///
/// let v2_chains = get_supported_v2_chains();
/// assert!(v2_chains.contains(&1)); // Ethereum
/// assert!(!v2_chains.contains(&80094)); // Berachain has no V2
/// ```
pub fn get_supported_v2_chains() -> Vec<u64> {
    use NamedChain::*;

    let all_chains = [
        Mainnet,
        Optimism,
        BinanceSmartChain,
        Polygon,
        Fraxtal,
        ZkSync,
        Unichain,
        Mantle,
        Base,
        Mode,
        Arbitrum,
        Avalanche,
        Linea,
        Scroll,
        Sonic,
    ];

    all_chains
        .iter()
        .filter_map(|&chain| {
            // Check if the chain supports V2 using trait method
            if chain.supports_v2() {
                Some(chain as u64)
            } else {
                None
            }
        })
        .collect()
}

/// Get all chain IDs that support V3 routers
///
/// # Returns
///
/// A vector of chain IDs that have V3 router deployments
///
/// # Example
///
/// ```rust
/// use odos_sdk::get_supported_v3_chains;
///
/// let v3_chains = get_supported_v3_chains();
/// assert!(v3_chains.contains(&1)); // Ethereum
/// assert!(v3_chains.contains(&80094)); // Berachain
/// ```
pub fn get_supported_v3_chains() -> Vec<u64> {
    use NamedChain::*;

    let all_chains = [
        Mainnet,
        Optimism,
        BinanceSmartChain,
        Polygon,
        Fraxtal,
        ZkSync,
        Unichain,
        Mantle,
        Base,
        Mode,
        Arbitrum,
        Avalanche,
        Linea,
        Scroll,
        Sonic,
        Berachain,
    ];

    all_chains
        .iter()
        .filter_map(|&chain| {
            // Check if the chain supports V3 using trait method
            if chain.supports_v3() {
                Some(chain as u64)
            } else {
                None
            }
        })
        .collect()
}

/// Check if both V2 and V3 are supported on a given chain
///
/// This function leverages the `OdosChain` trait to provide chain ID-based
/// lookups while maintaining a single source of truth for chain support.
///
/// # Arguments
///
/// * `chain_id` - The chain ID to check
///
/// # Returns
///
/// * `Some((v2_address, v3_address))` - Both addresses if the chain is supported
/// * `None` - If the chain is not supported by either version
///
/// # Example
///
/// ```rust
/// use odos_sdk::get_both_router_addresses;
///
/// if let Some((v2_addr, v3_addr)) = get_both_router_addresses(1) {
///     println!("Ethereum - V2: {v2_addr}, V3: {v3_addr}");
/// }
/// ```
pub fn get_both_router_addresses(chain_id: u64) -> Option<(Address, Address)> {
    match (
        get_v2_router_by_chain_id(chain_id),
        get_v3_router_by_chain_id(chain_id),
    ) {
        (Some(v2_addr), Some(v3_addr)) => Some((v2_addr, v3_addr)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Address;

    use super::*;

    #[test]
    fn test_all_addresses_are_valid_hex() {
        let addresses = [
            ODOS_V2_ARBITRUM_ROUTER,
            ODOS_V2_BASE_ROUTER,
            ODOS_V2_BSC_ROUTER,
            ODOS_V2_ETHEREUM_ROUTER,
            ODOS_V2_OP_ROUTER,
            ODOS_V2_AVALANCHE_ROUTER,
            ODOS_V2_POLYGON_ROUTER,
            ODOS_V2_FRAXTAL_ROUTER,
            ODOS_V2_LINEA_ROUTER,
            ODOS_V2_MANTLE_ROUTER,
            ODOS_V2_MODE_ROUTER,
            ODOS_V2_SCROLL_ROUTER,
            ODOS_V2_SONIC_ROUTER,
            ODOS_V2_ZKSYNC_ROUTER,
            ODOS_V2_UNICHAIN_ROUTER,
            ODOS_V3,
        ];

        for address in addresses {
            assert!(address != Address::ZERO, "Invalid address: {address}",);
        }
    }

    #[test]
    fn test_trait_and_utility_functions_agree() {
        // Test that trait-based and utility function approaches give same results
        let test_chains = [
            (1, NamedChain::Mainnet),
            (42161, NamedChain::Arbitrum),
            (137, NamedChain::Polygon),
        ];

        for (chain_id, named_chain) in test_chains {
            // Both should agree on support
            assert_eq!(
                named_chain.supports_odos(),
                get_v2_router_by_chain_id(chain_id).is_some()
            );
            assert_eq!(
                named_chain.supports_v3(),
                get_v3_router_by_chain_id(chain_id).is_some()
            );

            // Both should return the same addresses
            if let Some(v2_addr_str) = get_v2_router_by_chain_id(chain_id) {
                if let Ok(v2_addr_trait) = named_chain.v2_router_address() {
                    assert_eq!(v2_addr_trait, v2_addr_str);
                }
            }

            if let Some(v3_addr_str) = get_v3_router_by_chain_id(chain_id) {
                if let Ok(v3_addr_trait) = named_chain.v3_router_address() {
                    assert_eq!(v3_addr_trait, v3_addr_str);
                }
            }
        }
    }

    #[test]
    fn test_chain_id_lookup() {
        assert_eq!(get_v2_router_by_chain_id(1), Some(ODOS_V2_ETHEREUM_ROUTER));
        assert_eq!(
            get_v2_router_by_chain_id(42161),
            Some(ODOS_V2_ARBITRUM_ROUTER)
        );
        assert_eq!(get_v2_router_by_chain_id(999999), None);
    }

    #[test]
    fn test_both_router_addresses() {
        // Test that we can get both addresses for supported chains
        let (v2_addr, v3_addr) = get_both_router_addresses(1).unwrap();
        assert_eq!(v2_addr, ODOS_V2_ETHEREUM_ROUTER);
        assert_eq!(v3_addr, ODOS_V3);

        // Test that unsupported chains return None
        assert_eq!(get_both_router_addresses(999999), None);
    }

    #[test]
    fn test_chain_id_conversion() {
        // Test that NamedChain::try_from works as expected
        assert_eq!(NamedChain::try_from(1u64), Ok(NamedChain::Mainnet));
        assert_eq!(NamedChain::try_from(42161u64), Ok(NamedChain::Arbitrum));
        assert!(NamedChain::try_from(999999u64).is_err());
    }

    #[test]
    fn test_utility_functions_use_standard_conversion() {
        // Test that our utility functions work with the standard TryFrom
        assert!(get_v2_router_by_chain_id(1).is_some());
        assert!(get_v3_router_by_chain_id(1).is_some());

        // Test with unsupported chain ID
        assert!(get_v2_router_by_chain_id(999999).is_none());
        assert!(get_v3_router_by_chain_id(999999).is_none());
    }

    #[test]
    fn test_berachain_router_availability() {
        let berachain_chain_id = NamedChain::Berachain as u64;

        // Berachain should have LO router
        assert!(
            get_lo_router_by_chain_id(berachain_chain_id).is_some(),
            "Berachain should have LO router"
        );
        assert_eq!(
            get_lo_router_by_chain_id(berachain_chain_id).unwrap(),
            ODOS_LO_BERACHAIN_ROUTER
        );

        // Berachain should NOT have V2 router
        assert!(
            get_v2_router_by_chain_id(berachain_chain_id).is_none(),
            "Berachain should NOT have V2 router"
        );

        // Berachain should have V3 router
        assert!(
            get_v3_router_by_chain_id(berachain_chain_id).is_some(),
            "Berachain should have V3 router"
        );
        assert_eq!(
            get_v3_router_by_chain_id(berachain_chain_id).unwrap(),
            ODOS_V3
        );
    }

    #[test]
    fn test_berachain_in_supported_chains_lists() {
        let berachain_chain_id = NamedChain::Berachain as u64;

        // Berachain should be in general supported chains
        let all_chains = get_supported_chains();
        assert!(
            all_chains.contains(&berachain_chain_id),
            "Berachain should be in supported chains list"
        );

        // Berachain should be in LO chains
        let lo_chains = get_supported_lo_chains();
        assert!(
            lo_chains.contains(&berachain_chain_id),
            "Berachain should be in LO chains list"
        );

        // Berachain should NOT be in V2 chains
        let v2_chains = get_supported_v2_chains();
        assert!(
            !v2_chains.contains(&berachain_chain_id),
            "Berachain should NOT be in V2 chains list"
        );

        // Berachain should be in V3 chains
        let v3_chains = get_supported_v3_chains();
        assert!(
            v3_chains.contains(&berachain_chain_id),
            "Berachain should be in V3 chains list"
        );
    }

    #[test]
    fn test_supported_chains_consistency() {
        // Every chain in supported_chains should have at least one router
        let all_chains = get_supported_chains();
        let lo_chains = get_supported_lo_chains();
        let v2_chains = get_supported_v2_chains();
        let v3_chains = get_supported_v3_chains();

        for &chain_id in &all_chains {
            let has_lo = lo_chains.contains(&chain_id);
            let has_v2 = v2_chains.contains(&chain_id);
            let has_v3 = v3_chains.contains(&chain_id);

            assert!(
                has_lo || has_v2 || has_v3,
                "Chain {} should have at least one router type",
                chain_id
            );
        }
    }

    #[test]
    fn test_all_lo_addresses_are_valid() {
        let addresses = [
            ODOS_LO_ETHEREUM_ROUTER,
            ODOS_LO_OP_ROUTER,
            ODOS_LO_BSC_ROUTER,
            ODOS_LO_POLYGON_ROUTER,
            ODOS_LO_BERACHAIN_ROUTER,
            ODOS_LO_ARBITRUM_ROUTER,
            ODOS_LO_AVALANCHE_ROUTER,
            ODOS_LO_BASE_ROUTER,
            ODOS_LO_FRAXTAL_ROUTER,
            ODOS_LO_LINEA_ROUTER,
            ODOS_LO_MANTLE_ROUTER,
            ODOS_LO_MODE_ROUTER,
            ODOS_LO_SCROLL_ROUTER,
            ODOS_LO_SONIC_ROUTER,
            ODOS_LO_ZKSYNC_ROUTER,
            ODOS_LO_UNICHAIN_ROUTER,
        ];

        for address in addresses {
            assert!(address != Address::ZERO, "Invalid LO address: {address}");
        }
    }
}
