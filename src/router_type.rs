//! Router type definitions for Odos protocol
//!
//! This module provides enums and types to represent the different router types
//! available across Odos-supported chains.

use std::fmt;

/// Represents the different types of Odos routers
///
/// Different chains support different combinations of these router types.
/// Use the `OdosChain` trait methods to check router availability per chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouterType {
    /// Limit Order V2 router for limit order functionality
    ///
    /// Available on: Ethereum, Optimism, BSC, Polygon, Berachain
    LimitOrder,

    /// V2 router for swap functionality
    ///
    /// Available on all chains except Berachain
    V2,

    /// V3 router for enhanced swap functionality
    ///
    /// Available on all supported chains (unified address)
    V3,
}

impl RouterType {
    /// Returns all possible router types
    pub const fn all() -> [RouterType; 3] {
        [RouterType::LimitOrder, RouterType::V2, RouterType::V3]
    }

    /// Returns the router type as a string identifier
    pub const fn as_str(&self) -> &'static str {
        match self {
            RouterType::LimitOrder => "LO",
            RouterType::V2 => "V2",
            RouterType::V3 => "V3",
        }
    }
}

impl fmt::Display for RouterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents which routers are available on a specific chain
///
/// This provides a type-safe way to query router availability without
/// needing to call multiple trait methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouterAvailability {
    /// Whether the Limit Order V2 router is available
    pub limit_order: bool,
    /// Whether the V2 router is available
    pub v2: bool,
    /// Whether the V3 router is available
    pub v3: bool,
}

impl RouterAvailability {
    /// Creates a new `RouterAvailability` with all routers available
    pub const fn all() -> Self {
        Self {
            limit_order: true,
            v2: true,
            v3: true,
        }
    }

    /// Creates a new `RouterAvailability` with no routers available
    pub const fn none() -> Self {
        Self {
            limit_order: false,
            v2: false,
            v3: false,
        }
    }

    /// Creates availability for LO + V3 only (like Berachain)
    pub const fn lo_v3_only() -> Self {
        Self {
            limit_order: true,
            v2: false,
            v3: true,
        }
    }

    /// Creates availability for V2 + V3 only (most chains)
    pub const fn v2_v3_only() -> Self {
        Self {
            limit_order: false,
            v2: true,
            v3: true,
        }
    }

    /// Checks if the specified router type is available
    pub const fn has(&self, router_type: RouterType) -> bool {
        match router_type {
            RouterType::LimitOrder => self.limit_order,
            RouterType::V2 => self.v2,
            RouterType::V3 => self.v3,
        }
    }

    /// Returns all available router types
    pub fn available_routers(&self) -> Vec<RouterType> {
        let mut routers = Vec::new();
        if self.limit_order {
            routers.push(RouterType::LimitOrder);
        }
        if self.v2 {
            routers.push(RouterType::V2);
        }
        if self.v3 {
            routers.push(RouterType::V3);
        }
        routers
    }

    /// Returns the count of available routers
    pub const fn count(&self) -> usize {
        let mut count = 0;
        if self.limit_order {
            count += 1;
        }
        if self.v2 {
            count += 1;
        }
        if self.v3 {
            count += 1;
        }
        count
    }

    /// Checks if any router is available
    pub const fn has_any(&self) -> bool {
        self.limit_order || self.v2 || self.v3
    }
}

impl fmt::Display for RouterAvailability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let routers = self.available_routers();
        if routers.is_empty() {
            write!(f, "No routers available")
        } else {
            write!(
                f,
                "{}",
                routers
                    .iter()
                    .map(|r| r.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_type_all() {
        let all = RouterType::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&RouterType::LimitOrder));
        assert!(all.contains(&RouterType::V2));
        assert!(all.contains(&RouterType::V3));
    }

    #[test]
    fn test_router_type_display() {
        assert_eq!(RouterType::LimitOrder.to_string(), "LO");
        assert_eq!(RouterType::V2.to_string(), "V2");
        assert_eq!(RouterType::V3.to_string(), "V3");
    }

    #[test]
    fn test_router_availability_all() {
        let avail = RouterAvailability::all();
        assert!(avail.limit_order);
        assert!(avail.v2);
        assert!(avail.v3);
        assert_eq!(avail.count(), 3);
    }

    #[test]
    fn test_router_availability_none() {
        let avail = RouterAvailability::none();
        assert!(!avail.limit_order);
        assert!(!avail.v2);
        assert!(!avail.v3);
        assert_eq!(avail.count(), 0);
        assert!(!avail.has_any());
    }

    #[test]
    fn test_router_availability_lo_v3_only() {
        let avail = RouterAvailability::lo_v3_only();
        assert!(avail.limit_order);
        assert!(!avail.v2);
        assert!(avail.v3);
        assert_eq!(avail.count(), 2);
        assert!(avail.has(RouterType::LimitOrder));
        assert!(!avail.has(RouterType::V2));
        assert!(avail.has(RouterType::V3));
    }

    #[test]
    fn test_router_availability_v2_v3_only() {
        let avail = RouterAvailability::v2_v3_only();
        assert!(!avail.limit_order);
        assert!(avail.v2);
        assert!(avail.v3);
        assert_eq!(avail.count(), 2);
    }

    #[test]
    fn test_available_routers() {
        let avail = RouterAvailability::all();
        let routers = avail.available_routers();
        assert_eq!(routers.len(), 3);

        let avail = RouterAvailability::lo_v3_only();
        let routers = avail.available_routers();
        assert_eq!(routers.len(), 2);
        assert!(routers.contains(&RouterType::LimitOrder));
        assert!(routers.contains(&RouterType::V3));
    }

    #[test]
    fn test_display() {
        let avail = RouterAvailability::all();
        assert_eq!(avail.to_string(), "LO, V2, V3");

        let avail = RouterAvailability::lo_v3_only();
        assert_eq!(avail.to_string(), "LO, V3");

        let avail = RouterAvailability::none();
        assert_eq!(avail.to_string(), "No routers available");
    }
}
