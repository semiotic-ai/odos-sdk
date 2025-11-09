use std::fmt;

use alloy_chains::NamedChain;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{OdosChain, OdosChainError, OdosChainResult};

/// Type-safe chain identifier with convenient constructors
///
/// Provides ergonomic helpers for accessing supported chains while
/// maintaining full compatibility with `alloy_chains::NamedChain`.
///
/// # Examples
///
/// ```rust
/// use odos_sdk::Chain;
///
/// // Convenient constructors
/// let chain = Chain::ethereum();
/// let chain = Chain::arbitrum();
/// let chain = Chain::base();
///
/// // From chain ID
/// let chain = Chain::from_chain_id(1)?;  // Ethereum
/// let chain = Chain::from_chain_id(42161)?;  // Arbitrum
///
/// // Access inner NamedChain
/// let named = chain.inner();
///
/// // Use OdosChain trait methods
/// let router = chain.v3_router_address()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Chain(NamedChain);

impl Chain {
    /// Ethereum Mainnet (Chain ID: 1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::ethereum();
    /// assert_eq!(chain.id(), 1);
    /// ```
    pub const fn ethereum() -> Self {
        Self(NamedChain::Mainnet)
    }

    /// Arbitrum One (Chain ID: 42161)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::arbitrum();
    /// assert_eq!(chain.id(), 42161);
    /// ```
    pub const fn arbitrum() -> Self {
        Self(NamedChain::Arbitrum)
    }

    /// Optimism (Chain ID: 10)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::optimism();
    /// assert_eq!(chain.id(), 10);
    /// ```
    pub const fn optimism() -> Self {
        Self(NamedChain::Optimism)
    }

    /// Polygon (Chain ID: 137)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::polygon();
    /// assert_eq!(chain.id(), 137);
    /// ```
    pub const fn polygon() -> Self {
        Self(NamedChain::Polygon)
    }

    /// Base (Chain ID: 8453)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::base();
    /// assert_eq!(chain.id(), 8453);
    /// ```
    pub const fn base() -> Self {
        Self(NamedChain::Base)
    }

    /// BNB Smart Chain (Chain ID: 56)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::bsc();
    /// assert_eq!(chain.id(), 56);
    /// ```
    pub const fn bsc() -> Self {
        Self(NamedChain::BinanceSmartChain)
    }

    /// Avalanche C-Chain (Chain ID: 43114)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::avalanche();
    /// assert_eq!(chain.id(), 43114);
    /// ```
    pub const fn avalanche() -> Self {
        Self(NamedChain::Avalanche)
    }

    /// Linea (Chain ID: 59144)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::linea();
    /// assert_eq!(chain.id(), 59144);
    /// ```
    pub const fn linea() -> Self {
        Self(NamedChain::Linea)
    }

    /// Scroll (Chain ID: 534352)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::scroll();
    /// assert_eq!(chain.id(), 534352);
    /// ```
    pub const fn scroll() -> Self {
        Self(NamedChain::Scroll)
    }

    /// ZkSync Era (Chain ID: 324)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::zksync();
    /// assert_eq!(chain.id(), 324);
    /// ```
    pub const fn zksync() -> Self {
        Self(NamedChain::ZkSync)
    }

    /// Mantle (Chain ID: 5000)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::mantle();
    /// assert_eq!(chain.id(), 5000);
    /// ```
    pub const fn mantle() -> Self {
        Self(NamedChain::Mantle)
    }

    /// Mode (Chain ID: 34443)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::mode();
    /// assert_eq!(chain.id(), 34443);
    /// ```
    pub const fn mode() -> Self {
        Self(NamedChain::Mode)
    }

    /// Fraxtal (Chain ID: 252)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::fraxtal();
    /// assert_eq!(chain.id(), 252);
    /// ```
    pub const fn fraxtal() -> Self {
        Self(NamedChain::Fraxtal)
    }

    /// Sonic (Chain ID: 146)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::sonic();
    /// assert_eq!(chain.id(), 146);
    /// ```
    pub const fn sonic() -> Self {
        Self(NamedChain::Sonic)
    }

    /// Berachain (Chain ID: 80094)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::berachain();
    /// assert_eq!(chain.id(), 80094);
    /// ```
    pub const fn berachain() -> Self {
        Self(NamedChain::Berachain)
    }

    /// Unichain (Chain ID: 130)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::unichain();
    /// assert_eq!(chain.id(), 130);
    /// ```
    pub const fn unichain() -> Self {
        Self(NamedChain::Unichain)
    }

    /// Create a chain from a chain ID
    ///
    /// # Arguments
    ///
    /// * `id` - The EVM chain ID
    ///
    /// # Returns
    ///
    /// * `Ok(Chain)` - If the chain ID is recognized
    /// * `Err(OdosChainError)` - If the chain ID is not supported
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// let chain = Chain::from_chain_id(1)?;      // Ethereum
    /// let chain = Chain::from_chain_id(42161)?;  // Arbitrum
    /// let chain = Chain::from_chain_id(8453)?;   // Base
    ///
    /// // Unsupported chain
    /// assert!(Chain::from_chain_id(999999).is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_chain_id(id: u64) -> OdosChainResult<Self> {
        NamedChain::try_from(id)
            .map(Self)
            .map_err(|_| OdosChainError::UnsupportedChain {
                chain: format!("Chain ID {id}"),
            })
    }

    /// Get the chain ID
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    ///
    /// assert_eq!(Chain::ethereum().id(), 1);
    /// assert_eq!(Chain::arbitrum().id(), 42161);
    /// assert_eq!(Chain::base().id(), 8453);
    /// ```
    pub fn id(&self) -> u64 {
        self.0.into()
    }

    /// Get the inner `NamedChain`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Chain;
    /// use alloy_chains::NamedChain;
    ///
    /// let chain = Chain::ethereum();
    /// assert_eq!(chain.inner(), NamedChain::Mainnet);
    /// ```
    pub const fn inner(&self) -> NamedChain {
        self.0
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<NamedChain> for Chain {
    fn from(chain: NamedChain) -> Self {
        Self(chain)
    }
}

impl From<Chain> for NamedChain {
    fn from(chain: Chain) -> Self {
        chain.0
    }
}

impl From<Chain> for u64 {
    fn from(chain: Chain) -> Self {
        chain.0.into()
    }
}

// Custom serialization using chain ID
impl Serialize for Chain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let chain_id: u64 = self.0.into();
        chain_id.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Chain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let chain_id = u64::deserialize(deserializer)?;
        NamedChain::try_from(chain_id)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

// Implement OdosChain trait by delegating to inner NamedChain
impl OdosChain for Chain {
    fn lo_router_address(&self) -> OdosChainResult<alloy_primitives::Address> {
        self.0.lo_router_address()
    }

    fn v2_router_address(&self) -> OdosChainResult<alloy_primitives::Address> {
        self.0.v2_router_address()
    }

    fn v3_router_address(&self) -> OdosChainResult<alloy_primitives::Address> {
        self.0.v3_router_address()
    }

    fn supports_odos(&self) -> bool {
        self.0.supports_odos()
    }

    fn supports_lo(&self) -> bool {
        self.0.supports_lo()
    }

    fn supports_v2(&self) -> bool {
        self.0.supports_v2()
    }

    fn supports_v3(&self) -> bool {
        self.0.supports_v3()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_constructors() {
        assert_eq!(Chain::ethereum().id(), 1);
        assert_eq!(Chain::arbitrum().id(), 42161);
        assert_eq!(Chain::optimism().id(), 10);
        assert_eq!(Chain::polygon().id(), 137);
        assert_eq!(Chain::base().id(), 8453);
        assert_eq!(Chain::bsc().id(), 56);
        assert_eq!(Chain::avalanche().id(), 43114);
        assert_eq!(Chain::linea().id(), 59144);
        assert_eq!(Chain::scroll().id(), 534352);
        assert_eq!(Chain::zksync().id(), 324);
        assert_eq!(Chain::mantle().id(), 5000);
        assert_eq!(Chain::mode().id(), 34443);
        assert_eq!(Chain::fraxtal().id(), 252);
        assert_eq!(Chain::sonic().id(), 146);
        assert_eq!(Chain::berachain().id(), 80094);
        assert_eq!(Chain::unichain().id(), 130);
    }

    #[test]
    fn test_from_chain_id() {
        assert_eq!(Chain::from_chain_id(1).unwrap().id(), 1);
        assert_eq!(Chain::from_chain_id(42161).unwrap().id(), 42161);
        assert_eq!(Chain::from_chain_id(8453).unwrap().id(), 8453);

        // Unsupported chain
        assert!(Chain::from_chain_id(999999).is_err());
    }

    #[test]
    fn test_inner() {
        assert_eq!(Chain::ethereum().inner(), NamedChain::Mainnet);
        assert_eq!(Chain::arbitrum().inner(), NamedChain::Arbitrum);
        assert_eq!(Chain::base().inner(), NamedChain::Base);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Chain::ethereum()), "mainnet");
        assert_eq!(format!("{}", Chain::arbitrum()), "arbitrum");
        assert_eq!(format!("{}", Chain::base()), "base");
    }

    #[test]
    fn test_conversions() {
        // From NamedChain
        let chain: Chain = NamedChain::Mainnet.into();
        assert_eq!(chain.id(), 1);

        // To NamedChain
        let named: NamedChain = Chain::ethereum().into();
        assert_eq!(named, NamedChain::Mainnet);

        // To u64
        let id: u64 = Chain::ethereum().into();
        assert_eq!(id, 1);
    }

    #[test]
    fn test_odos_chain_trait() {
        let chain = Chain::ethereum();

        // Test trait methods work
        assert!(chain.supports_odos());
        assert!(chain.supports_v2());
        assert!(chain.supports_v3());
        assert!(chain.v2_router_address().is_ok());
        assert!(chain.v3_router_address().is_ok());
    }

    #[test]
    fn test_equality() {
        assert_eq!(Chain::ethereum(), Chain::ethereum());
        assert_ne!(Chain::ethereum(), Chain::arbitrum());
    }

    #[test]
    fn test_serialization() {
        let chain = Chain::ethereum();

        // Serialize (as chain ID)
        let json = serde_json::to_string(&chain).unwrap();
        assert_eq!(json, "1"); // Ethereum chain ID

        // Deserialize
        let deserialized: Chain = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, chain);

        // Test other chains
        assert_eq!(serde_json::to_string(&Chain::arbitrum()).unwrap(), "42161");
        assert_eq!(serde_json::to_string(&Chain::base()).unwrap(), "8453");
    }
}
