use alloy_chains::NamedChain;
use alloy_primitives::Address;
use thiserror::Error;

use crate::{
    ODOS_V2_ARBITRUM_ROUTER, ODOS_V2_AVALANCHE_ROUTER, ODOS_V2_BASE_ROUTER, ODOS_V2_BSC_ROUTER,
    ODOS_V2_ETHEREUM_ROUTER, ODOS_V2_FRAXTAL_ROUTER, ODOS_V2_LINEA_ROUTER, ODOS_V2_MANTLE_ROUTER,
    ODOS_V2_MODE_ROUTER, ODOS_V2_OP_ROUTER, ODOS_V2_POLYGON_ROUTER, ODOS_V2_SCROLL_ROUTER,
    ODOS_V2_SONIC_ROUTER, ODOS_V2_UNICHAIN_ROUTER, ODOS_V2_ZKSYNC_ROUTER, ODOS_V3,
};

/// Errors that can occur when working with Odos chains
#[derive(Error, Debug, Clone, PartialEq)]
pub enum OdosChainError {
    /// The chain is not supported by Odos protocol
    #[error("Chain {chain:?} is not supported by Odos protocol")]
    UnsupportedChain { chain: String },

    /// The V2 router is not available on this chain
    #[error("Odos V2 router is not available on chain {chain:?}")]
    V2NotAvailable { chain: String },

    /// The V3 router is not available on this chain  
    #[error("Odos V3 router is not available on chain {chain:?}")]
    V3NotAvailable { chain: String },

    /// Invalid address format
    #[error("Invalid address format: {address}")]
    InvalidAddress { address: String },
}

/// Result type for Odos chain operations
pub type OdosChainResult<T> = Result<T, OdosChainError>;

/// Trait for chains that support Odos protocol
///
/// This trait provides a type-safe way to access Odos router addresses
/// for supported blockchain networks, integrating seamlessly with the
/// Alloy ecosystem.
///
/// # Examples
///
/// ```rust
/// use odos_sdk::OdosChain;
/// use alloy_chains::NamedChain;
///
/// // Get V2 router address
/// let v2_router = NamedChain::Mainnet.v2_router_address()?;
///
/// // Get V3 router address
/// let v3_router = NamedChain::Mainnet.v3_router_address()?;
///
/// // Get both addresses
/// let (v2, v3) = NamedChain::Arbitrum.both_router_addresses()?;
///
/// // Check support
/// assert!(NamedChain::Mainnet.supports_odos());
/// assert!(NamedChain::Mainnet.supports_v3());
/// # Ok::<(), odos_sdk::OdosChainError>(())
/// ```
pub trait OdosChain {
    /// Get the V2 router address for this chain
    ///
    /// # Returns
    ///
    /// * `Ok(Address)` - The V2 router contract address
    /// * `Err(OdosChainError)` - If the chain is not supported or address is invalid
    ///
    /// # Example
    ///
    /// ```rust
    /// use odos_sdk::OdosChain;
    /// use alloy_chains::NamedChain;
    ///
    /// let address = NamedChain::Mainnet.v2_router_address()?;
    /// # Ok::<(), odos_sdk::OdosChainError>(())
    /// ```
    fn v2_router_address(&self) -> OdosChainResult<Address>;

    /// Get the V3 router address for this chain
    ///
    /// V3 uses the same address across all supported chains,
    /// following CREATE2 deterministic deployment.
    ///
    /// # Returns
    ///
    /// * `Ok(Address)` - The V3 router contract address
    /// * `Err(OdosChainError)` - If the chain is not supported or address is invalid
    ///
    /// # Example
    ///
    /// ```rust
    /// use odos_sdk::OdosChain;
    /// use alloy_chains::NamedChain;
    ///
    /// let address = NamedChain::Mainnet.v3_router_address()?;
    /// # Ok::<(), odos_sdk::OdosChainError>(())
    /// ```
    fn v3_router_address(&self) -> OdosChainResult<Address>;

    /// Get both V2 and V3 router addresses for this chain
    ///
    /// # Returns
    ///
    /// * `Ok((v2_address, v3_address))` - Both router addresses
    /// * `Err(OdosChainError)` - If the chain is not supported by either version
    ///
    /// # Example
    ///
    /// ```rust
    /// use odos_sdk::OdosChain;
    /// use alloy_chains::NamedChain;
    ///
    /// let (v2, v3) = NamedChain::Arbitrum.both_router_addresses()?;
    /// # Ok::<(), odos_sdk::OdosChainError>(())
    /// ```
    fn both_router_addresses(&self) -> OdosChainResult<(Address, Address)> {
        Ok((self.v2_router_address()?, self.v3_router_address()?))
    }

    /// Check if this chain supports Odos protocol
    ///
    /// # Returns
    ///
    /// `true` if either V2 or V3 (or both) are supported on this chain
    fn supports_odos(&self) -> bool;

    /// Check if this chain supports Odos V2
    ///
    /// # Returns
    ///
    /// `true` if V2 is supported on this chain
    fn supports_v2(&self) -> bool;

    /// Check if this chain supports Odos V3
    ///
    /// # Returns
    ///
    /// `true` if V3 is supported on this chain
    fn supports_v3(&self) -> bool;

    /// Try to get the V2 router address without errors
    ///
    /// # Returns
    ///
    /// `Some(address)` if supported, `None` if not supported
    fn try_v2_router_address(&self) -> Option<Address> {
        self.v2_router_address().ok()
    }

    /// Try to get the V3 router address without errors
    ///
    /// # Returns
    ///
    /// `Some(address)` if supported, `None` if not supported
    fn try_v3_router_address(&self) -> Option<Address> {
        self.v3_router_address().ok()
    }

    /// Try to get both router addresses without errors
    ///
    /// # Returns
    ///
    /// `Some((v2_address, v3_address))` if both are supported, `None` otherwise
    fn try_both_router_addresses(&self) -> Option<(Address, Address)> {
        self.both_router_addresses().ok()
    }
}

impl OdosChain for NamedChain {
    fn v2_router_address(&self) -> OdosChainResult<Address> {
        use NamedChain::*;

        if !self.supports_odos() {
            return Err(OdosChainError::V2NotAvailable {
                chain: format!("{self:?}"),
            });
        }

        // If V2 is not available on this chain, fall back to V3
        if !self.supports_v2() {
            return self.v3_router_address();
        }

        Ok(match self {
            Arbitrum => ODOS_V2_ARBITRUM_ROUTER,
            Avalanche => ODOS_V2_AVALANCHE_ROUTER,
            Base => ODOS_V2_BASE_ROUTER,
            BinanceSmartChain => ODOS_V2_BSC_ROUTER,
            Fraxtal => ODOS_V2_FRAXTAL_ROUTER,
            Mainnet => ODOS_V2_ETHEREUM_ROUTER,
            Optimism => ODOS_V2_OP_ROUTER,
            Polygon => ODOS_V2_POLYGON_ROUTER,
            Linea => ODOS_V2_LINEA_ROUTER,
            Mantle => ODOS_V2_MANTLE_ROUTER,
            Mode => ODOS_V2_MODE_ROUTER,
            Scroll => ODOS_V2_SCROLL_ROUTER,
            Sonic => ODOS_V2_SONIC_ROUTER,
            ZkSync => ODOS_V2_ZKSYNC_ROUTER,
            Unichain => ODOS_V2_UNICHAIN_ROUTER,
            _ => {
                return Err(OdosChainError::UnsupportedChain {
                    chain: format!("{self:?}"),
                });
            }
        })
    }

    fn v3_router_address(&self) -> OdosChainResult<Address> {
        if !self.supports_odos() {
            return Err(OdosChainError::V3NotAvailable {
                chain: format!("{self:?}"),
            });
        }

        // If V3 is not available on this chain, fall back to V2
        if !self.supports_v3() {
            return self.v2_router_address();
        }

        Ok(ODOS_V3)
    }

    fn supports_odos(&self) -> bool {
        use NamedChain::*;
        matches!(
            self,
            Arbitrum
                | Avalanche
                | Base
                | Berachain
                | BinanceSmartChain
                | Fraxtal
                | Mainnet
                | Optimism
                | Polygon
                | Linea
                | Mantle
                | Mode
                | Scroll
                | Sonic
                | ZkSync
                | Unichain
        )
    }

    fn supports_v2(&self) -> bool {
        use NamedChain::*;
        matches!(
            self,
            Arbitrum
                | Avalanche
                | Base
                | BinanceSmartChain
                | Fraxtal
                | Mainnet
                | Optimism
                | Polygon
                | Linea
                | Mantle
                | Mode
                | Scroll
                | Sonic
                | ZkSync
                | Unichain
        )
    }

    fn supports_v3(&self) -> bool {
        use NamedChain::*;
        matches!(
            self,
            Arbitrum
                | Avalanche
                | Base
                | Berachain
                | BinanceSmartChain
                | Fraxtal
                | Mainnet
                | Optimism
                | Polygon
                | Linea
                | Mantle
                | Mode
                | Scroll
                | Sonic
                | ZkSync
                | Unichain
        )
    }
}

/// Extension trait for easy router selection
///
/// This trait provides convenient methods for choosing between V2 and V3
/// routers based on your requirements.
pub trait OdosRouterSelection: OdosChain {
    /// Get the recommended router address for this chain
    ///
    /// Currently defaults to V3 for enhanced features, but this
    /// may change based on performance characteristics.
    ///
    /// # Returns
    ///
    /// * `Ok(Address)` - The recommended router address
    /// * `Err(OdosChainError)` - If the chain is not supported
    ///
    /// # Example
    ///
    /// ```rust
    /// use odos_sdk::{OdosChain, OdosRouterSelection};
    /// use alloy_chains::NamedChain;
    ///
    /// let address = NamedChain::Base.recommended_router_address()?;
    /// # Ok::<(), odos_sdk::OdosChainError>(())
    /// ```
    fn recommended_router_address(&self) -> OdosChainResult<Address> {
        self.v3_router_address()
    }

    /// Get router address with fallback strategy
    ///
    /// Tries V3 first, falls back to V2 if needed.
    /// This is useful for maximum compatibility.
    ///
    /// # Returns
    ///
    /// * `Ok(Address)` - V3 address if available, otherwise V2 address
    /// * `Err(OdosChainError)` - If neither version is supported
    ///
    /// # Example
    ///
    /// ```rust
    /// use odos_sdk::{OdosChain, OdosRouterSelection};
    /// use alloy_chains::NamedChain;
    ///
    /// let address = NamedChain::Mainnet.router_address_with_fallback()?;
    /// # Ok::<(), odos_sdk::OdosChainError>(())
    /// ```
    fn router_address_with_fallback(&self) -> OdosChainResult<Address> {
        self.v3_router_address()
            .or_else(|_| self.v2_router_address())
    }

    /// Get router address based on preference
    ///
    /// # Arguments
    ///
    /// * `prefer_v3` - Whether to prefer V3 when both are available
    ///
    /// # Returns
    ///
    /// * `Ok(Address)` - The appropriate router address based on preference
    /// * `Err(OdosChainError)` - If the preferred version is not supported
    ///
    /// # Example
    ///
    /// ```rust
    /// use odos_sdk::{OdosChain, OdosRouterSelection};
    /// use alloy_chains::NamedChain;
    ///
    /// let v3_address = NamedChain::Mainnet.router_address_by_preference(true)?;
    /// let v2_address = NamedChain::Mainnet.router_address_by_preference(false)?;
    /// # Ok::<(), odos_sdk::OdosChainError>(())
    /// ```
    fn router_address_by_preference(&self, prefer_v3: bool) -> OdosChainResult<Address> {
        if prefer_v3 {
            self.v3_router_address()
        } else {
            self.v2_router_address()
        }
    }
}

impl<T: OdosChain> OdosRouterSelection for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_chains::NamedChain;

    #[test]
    fn test_v2_router_addresses() {
        let chains = [
            NamedChain::Mainnet,
            NamedChain::Arbitrum,
            NamedChain::Optimism,
            NamedChain::Polygon,
            NamedChain::Base,
        ];

        for chain in chains {
            let address = chain.v2_router_address().unwrap();
            assert!(address != Address::ZERO);
            assert_eq!(address.to_string().len(), 42); // 0x + 40 hex chars
        }
    }

    #[test]
    fn test_v3_router_addresses() {
        let chains = [
            NamedChain::Mainnet,
            NamedChain::Arbitrum,
            NamedChain::Optimism,
            NamedChain::Polygon,
            NamedChain::Base,
        ];

        for chain in chains {
            let address = chain.v3_router_address().unwrap();
            assert_eq!(address, ODOS_V3);
        }
    }

    #[test]
    fn test_both_router_addresses() {
        let (v2_addr, v3_addr) = NamedChain::Mainnet.both_router_addresses().unwrap();
        assert_eq!(v2_addr, ODOS_V2_ETHEREUM_ROUTER);
        assert_eq!(v3_addr, ODOS_V3);
    }

    #[test]
    fn test_supports_odos() {
        assert!(NamedChain::Mainnet.supports_odos());
        assert!(NamedChain::Arbitrum.supports_odos());
        assert!(NamedChain::Berachain.supports_odos());
        assert!(!NamedChain::Sepolia.supports_odos());
    }

    #[test]
    fn test_supports_v2() {
        assert!(NamedChain::Mainnet.supports_v2());
        assert!(NamedChain::Arbitrum.supports_v2());
        assert!(!NamedChain::Berachain.supports_v2()); // Berachain only has V3
        assert!(!NamedChain::Sepolia.supports_v2());
    }

    #[test]
    fn test_supports_v3() {
        assert!(NamedChain::Mainnet.supports_v3());
        assert!(NamedChain::Arbitrum.supports_v3());
        assert!(NamedChain::Berachain.supports_v3());
        assert!(!NamedChain::Sepolia.supports_v3());
    }

    #[test]
    fn test_berachain_v3_only() {
        // Berachain only has V3, not V2
        assert!(!NamedChain::Berachain.supports_v2());
        assert!(NamedChain::Berachain.supports_v3());

        // Requesting V2 should fallback to V3
        let v2_result = NamedChain::Berachain.v2_router_address();
        let v3_result = NamedChain::Berachain.v3_router_address();

        assert!(v2_result.is_ok());
        assert!(v3_result.is_ok());
        assert_eq!(v2_result.unwrap(), v3_result.unwrap());
    }

    #[test]
    fn test_try_methods() {
        assert!(NamedChain::Mainnet.try_v2_router_address().is_some());
        assert!(NamedChain::Mainnet.try_v3_router_address().is_some());
        assert!(NamedChain::Sepolia.try_v2_router_address().is_none());
        assert!(NamedChain::Sepolia.try_v3_router_address().is_none());

        assert!(NamedChain::Mainnet.try_both_router_addresses().is_some());
        assert!(NamedChain::Sepolia.try_both_router_addresses().is_none());
    }

    #[test]
    fn test_router_selection() {
        let chain = NamedChain::Mainnet;

        // Recommended should be V3
        assert_eq!(
            chain.recommended_router_address().unwrap(),
            chain.v3_router_address().unwrap()
        );

        // Fallback should also be V3 (since both are supported)
        assert_eq!(
            chain.router_address_with_fallback().unwrap(),
            chain.v3_router_address().unwrap()
        );

        // Preference-based selection
        assert_eq!(
            chain.router_address_by_preference(true).unwrap(),
            chain.v3_router_address().unwrap()
        );
        assert_eq!(
            chain.router_address_by_preference(false).unwrap(),
            chain.v2_router_address().unwrap()
        );
    }

    #[test]
    fn test_error_handling() {
        // Test unsupported chain
        let result = NamedChain::Sepolia.v2_router_address();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OdosChainError::V2NotAvailable { .. }
        ));

        let result = NamedChain::Sepolia.v3_router_address();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OdosChainError::V3NotAvailable { .. }
        ));
    }
}
