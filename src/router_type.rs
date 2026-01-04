// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! Router type definitions for Odos protocol
//!
//! This module provides enums and types to represent the different router types
//! available across Odos-supported chains.

use std::fmt;

/// Represents the different types of Odos routers.
///
/// Different chains support different combinations of these router types.
/// Use the `OdosChain` trait methods to check router availability per chain.
///
/// # Event Schemas
///
/// **Important:** Router types emit fundamentally different events:
///
/// | Router Type | Events Emitted |
/// |-------------|----------------|
/// | [`V2`](RouterType::V2) | `Swap`, `SwapMulti` |
/// | [`V3`](RouterType::V3) | `Swap`, `SwapMulti` (with referral/slippage fields) |
/// | [`LimitOrder`](RouterType::LimitOrder) | `LimitOrderFilled`, `LimitOrderCancelled`, etc. |
///
/// When iterating over router types for event processing, use [`swap_routers()`](RouterType::swap_routers)
/// to get only routers that emit `Swap`/`SwapMulti` events, or [`order_routers()`](RouterType::order_routers)
/// for limit order routers.
///
/// # Example
///
/// ```rust
/// use odos_sdk::RouterType;
///
/// // When processing swap events, iterate only over swap routers
/// for router_type in RouterType::swap_routers() {
///     // V2 and V3 both emit Swap/SwapMulti events
///     assert!(router_type.emits_swap_events());
/// }
///
/// // LimitOrder routers require separate event handling
/// for router_type in RouterType::order_routers() {
///     assert!(!router_type.emits_swap_events());
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouterType {
    /// Limit Order V2 router for limit order functionality.
    ///
    /// Available on all supported chains.
    ///
    /// **Note:** This router emits `LimitOrderFilled`, `LimitOrderCancelled`, and other
    /// limit order-specific events. It does **not** emit `Swap` or `SwapMulti` events
    /// like V2/V3 routers. Use [`emits_swap_events()`](RouterType::emits_swap_events) to
    /// check event compatibility, or [`swap_routers()`](RouterType::swap_routers) to
    /// iterate only over routers that emit swap events.
    LimitOrder,

    /// V2 router for swap functionality.
    ///
    /// Available on all supported chains.
    ///
    /// Emits `Swap` and `SwapMulti` events.
    V2,

    /// V3 router for enhanced swap functionality.
    ///
    /// Available on all supported chains (unified address via CREATE2).
    ///
    /// Emits `Swap` and `SwapMulti` events (with additional referral/slippage fields
    /// compared to V2).
    V3,
}

impl RouterType {
    /// Returns all possible router types.
    ///
    /// **Note:** This includes both swap routers (V2, V3) and order routers (LimitOrder),
    /// which emit different event types. For event processing, consider using
    /// [`swap_routers()`](RouterType::swap_routers) or [`order_routers()`](RouterType::order_routers)
    /// instead.
    pub const fn all() -> [RouterType; 3] {
        [RouterType::LimitOrder, RouterType::V2, RouterType::V3]
    }

    /// Returns router types that emit `Swap` and `SwapMulti` events.
    ///
    /// Use this when iterating over routers for swap event processing.
    /// Both V2 and V3 routers emit these events (with slightly different schemas).
    ///
    /// # Example
    ///
    /// ```rust
    /// use odos_sdk::RouterType;
    ///
    /// // Process only routers that emit swap events
    /// for router_type in RouterType::swap_routers() {
    ///     println!("Processing swap events for {:?}", router_type);
    /// }
    /// ```
    pub const fn swap_routers() -> [RouterType; 2] {
        [RouterType::V2, RouterType::V3]
    }

    /// Returns router types that emit limit order events (`LimitOrderFilled`, etc.).
    ///
    /// Use this when iterating over routers for limit order event processing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use odos_sdk::RouterType;
    ///
    /// // Process only routers that emit limit order events
    /// for router_type in RouterType::order_routers() {
    ///     println!("Processing order events for {:?}", router_type);
    /// }
    /// ```
    pub const fn order_routers() -> [RouterType; 1] {
        [RouterType::LimitOrder]
    }

    /// Returns whether this router type emits `Swap` and `SwapMulti` events.
    ///
    /// - `true` for [`V2`](RouterType::V2) and [`V3`](RouterType::V3)
    /// - `false` for [`LimitOrder`](RouterType::LimitOrder)
    ///
    /// # Example
    ///
    /// ```rust
    /// use odos_sdk::RouterType;
    ///
    /// assert!(RouterType::V2.emits_swap_events());
    /// assert!(RouterType::V3.emits_swap_events());
    /// assert!(!RouterType::LimitOrder.emits_swap_events());
    /// ```
    pub const fn emits_swap_events(&self) -> bool {
        matches!(self, RouterType::V2 | RouterType::V3)
    }

    /// Returns whether this router type emits limit order events.
    ///
    /// - `true` for [`LimitOrder`](RouterType::LimitOrder)
    /// - `false` for [`V2`](RouterType::V2) and [`V3`](RouterType::V3)
    ///
    /// # Example
    ///
    /// ```rust
    /// use odos_sdk::RouterType;
    ///
    /// assert!(RouterType::LimitOrder.emits_order_events());
    /// assert!(!RouterType::V2.emits_order_events());
    /// assert!(!RouterType::V3.emits_order_events());
    /// ```
    pub const fn emits_order_events(&self) -> bool {
        matches!(self, RouterType::LimitOrder)
    }

    /// Returns the router type as a string identifier.
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

    /// Creates availability for LO + V3 only
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

    #[test]
    fn test_swap_routers() {
        let swap = RouterType::swap_routers();
        assert_eq!(swap.len(), 2);
        assert!(swap.contains(&RouterType::V2));
        assert!(swap.contains(&RouterType::V3));
        assert!(!swap.contains(&RouterType::LimitOrder));
    }

    #[test]
    fn test_order_routers() {
        let order = RouterType::order_routers();
        assert_eq!(order.len(), 1);
        assert!(order.contains(&RouterType::LimitOrder));
        assert!(!order.contains(&RouterType::V2));
        assert!(!order.contains(&RouterType::V3));
    }

    #[test]
    fn test_emits_swap_events() {
        assert!(RouterType::V2.emits_swap_events());
        assert!(RouterType::V3.emits_swap_events());
        assert!(!RouterType::LimitOrder.emits_swap_events());
    }

    #[test]
    fn test_emits_order_events() {
        assert!(RouterType::LimitOrder.emits_order_events());
        assert!(!RouterType::V2.emits_order_events());
        assert!(!RouterType::V3.emits_order_events());
    }

    #[test]
    fn test_swap_and_order_routers_are_exhaustive() {
        // Verify that swap_routers + order_routers covers all router types
        let all: std::collections::HashSet<_> = RouterType::all().into_iter().collect();
        let swap: std::collections::HashSet<_> = RouterType::swap_routers().into_iter().collect();
        let order: std::collections::HashSet<_> = RouterType::order_routers().into_iter().collect();

        // Union of swap and order should equal all
        let combined: std::collections::HashSet<_> = swap.union(&order).copied().collect();
        assert_eq!(all, combined);

        // Swap and order should be disjoint
        assert!(swap.is_disjoint(&order));
    }
}
