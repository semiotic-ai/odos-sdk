// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use serde::{Deserialize, Serialize};

/// Type-safe referral code
///
/// Provides a clear, type-safe wrapper around referral codes with
/// convenient constants for common values.
///
/// # Examples
///
/// ```rust
/// use odos_sdk::ReferralCode;
///
/// // No referral code
/// let code = ReferralCode::NONE;
/// assert_eq!(code.code(), 0);
///
/// // Custom referral code
/// let code = ReferralCode::new(42);
/// assert_eq!(code.code(), 42);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ReferralCode(u32);

impl ReferralCode {
    /// No referral code (value: 0)
    ///
    /// This is the most common case - no referral program participation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::ReferralCode;
    ///
    /// let code = ReferralCode::NONE;
    /// assert_eq!(code.code(), 0);
    /// ```
    pub const NONE: Self = Self(0);

    /// Create a new referral code
    ///
    /// # Arguments
    ///
    /// * `code` - The referral code value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::ReferralCode;
    ///
    /// let code = ReferralCode::new(42);
    /// assert_eq!(code.code(), 42);
    ///
    /// let code = ReferralCode::new(1234);
    /// assert_eq!(code.code(), 1234);
    /// ```
    pub const fn new(code: u32) -> Self {
        Self(code)
    }

    /// Get the referral code value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::ReferralCode;
    ///
    /// let code = ReferralCode::new(42);
    /// assert_eq!(code.code(), 42);
    /// ```
    pub const fn code(&self) -> u32 {
        self.0
    }

    /// Check if this is the NONE referral code
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::ReferralCode;
    ///
    /// assert!(ReferralCode::NONE.is_none());
    /// assert!(!ReferralCode::new(42).is_none());
    /// ```
    pub const fn is_none(&self) -> bool {
        self.0 == 0
    }

    /// Check if this is a valid referral code (non-zero)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::ReferralCode;
    ///
    /// assert!(!ReferralCode::NONE.is_some());
    /// assert!(ReferralCode::new(42).is_some());
    /// ```
    pub const fn is_some(&self) -> bool {
        self.0 != 0
    }
}

impl Default for ReferralCode {
    /// Default referral code is NONE (0)
    fn default() -> Self {
        Self::NONE
    }
}

impl fmt::Display for ReferralCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_none() {
            write!(f, "None")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl From<ReferralCode> for u32 {
    fn from(code: ReferralCode) -> Self {
        code.0
    }
}

impl From<u32> for ReferralCode {
    fn from(code: u32) -> Self {
        Self(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_constant() {
        assert_eq!(ReferralCode::NONE.code(), 0);
        assert!(ReferralCode::NONE.is_none());
        assert!(!ReferralCode::NONE.is_some());
    }

    #[test]
    fn test_new() {
        let code = ReferralCode::new(42);
        assert_eq!(code.code(), 42);
        assert!(!code.is_none());
        assert!(code.is_some());

        let code = ReferralCode::new(0);
        assert_eq!(code.code(), 0);
        assert!(code.is_none());
        assert!(!code.is_some());

        let code = ReferralCode::new(u32::MAX);
        assert_eq!(code.code(), u32::MAX);
        assert!(code.is_some());
    }

    #[test]
    fn test_default() {
        let code = ReferralCode::default();
        assert_eq!(code, ReferralCode::NONE);
        assert_eq!(code.code(), 0);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", ReferralCode::NONE), "None");
        assert_eq!(format!("{}", ReferralCode::new(42)), "42");
        assert_eq!(format!("{}", ReferralCode::new(1234)), "1234");
    }

    #[test]
    fn test_conversions() {
        // From ReferralCode to u32
        let code = ReferralCode::new(42);
        let value: u32 = code.into();
        assert_eq!(value, 42);

        // From u32 to ReferralCode
        let code: ReferralCode = 42u32.into();
        assert_eq!(code.code(), 42);
    }

    #[test]
    fn test_equality() {
        assert_eq!(ReferralCode::new(42), ReferralCode::new(42));
        assert_ne!(ReferralCode::new(42), ReferralCode::new(43));
        assert_eq!(ReferralCode::NONE, ReferralCode::new(0));
    }

    #[test]
    fn test_ordering() {
        assert!(ReferralCode::new(1) < ReferralCode::new(2));
        assert!(ReferralCode::new(100) > ReferralCode::new(50));
        assert!(ReferralCode::NONE < ReferralCode::new(1));
    }

    #[test]
    fn test_serialization() {
        let code = ReferralCode::new(42);

        // Serialize
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, "42");

        // Deserialize
        let deserialized: ReferralCode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code(), 42);

        // Serialize NONE
        let json = serde_json::to_string(&ReferralCode::NONE).unwrap();
        assert_eq!(json, "0");
    }
}
