// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use serde::{Deserialize, Serialize};

/// Type-safe slippage percentage with validation
///
/// Ensures slippage values are within valid ranges and provides convenient
/// constructors for both percentage and basis point representations.
///
/// # Examples
///
/// ```rust
/// use odos_sdk::Slippage;
///
/// // Create from percentage (0.5% slippage)
/// let slippage = Slippage::percent(0.5)?;
/// assert_eq!(slippage.as_percent(), 0.5);
///
/// // Create from basis points (50 bps = 0.5%)
/// let slippage = Slippage::bps(50)?;
/// assert_eq!(slippage.as_percent(), 0.5);
/// assert_eq!(slippage.as_bps(), 50);
///
/// // Validation prevents invalid values
/// assert!(Slippage::percent(150.0).is_err());  // > 100%
/// assert!(Slippage::percent(-1.0).is_err());   // < 0%
/// assert!(Slippage::bps(15000).is_err());      // > 10000 bps
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Slippage(f64);

impl Slippage {
    /// Create slippage from percentage value
    ///
    /// # Arguments
    ///
    /// * `percent` - Slippage as percentage (e.g., 0.5 for 0.5%, 1.0 for 1%)
    ///
    /// # Returns
    ///
    /// * `Ok(Slippage)` - Valid slippage value
    /// * `Err(String)` - If percentage is < 0 or > 100
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Slippage;
    ///
    /// let slippage = Slippage::percent(0.5)?;  // 0.5%
    /// assert_eq!(slippage.as_percent(), 0.5);
    ///
    /// let slippage = Slippage::percent(1.0)?;  // 1%
    /// assert_eq!(slippage.as_percent(), 1.0);
    ///
    /// // Validation
    /// assert!(Slippage::percent(150.0).is_err());
    /// assert!(Slippage::percent(-0.1).is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn percent(percent: f64) -> Result<Self, String> {
        if percent < 0.0 {
            return Err(format!("Slippage percentage cannot be negative: {percent}"));
        }
        if percent > 100.0 {
            return Err(format!("Slippage percentage cannot exceed 100%: {percent}"));
        }
        Ok(Self(percent))
    }

    /// Create slippage from basis points
    ///
    /// # Arguments
    ///
    /// * `bps` - Slippage in basis points (e.g., 50 for 0.5%, 100 for 1%)
    ///
    /// # Returns
    ///
    /// * `Ok(Slippage)` - Valid slippage value
    /// * `Err(String)` - If basis points > 10000 (100%)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Slippage;
    ///
    /// let slippage = Slippage::bps(50)?;   // 50 bps = 0.5%
    /// assert_eq!(slippage.as_bps(), 50);
    /// assert_eq!(slippage.as_percent(), 0.5);
    ///
    /// let slippage = Slippage::bps(100)?;  // 100 bps = 1%
    /// assert_eq!(slippage.as_bps(), 100);
    /// assert_eq!(slippage.as_percent(), 1.0);
    ///
    /// // Validation
    /// assert!(Slippage::bps(15000).is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn bps(bps: u16) -> Result<Self, String> {
        if bps > 10000 {
            return Err(format!(
                "Slippage basis points cannot exceed 10000 (100%): {bps}"
            ));
        }
        Ok(Self(bps as f64 / 100.0))
    }

    /// Get slippage value as percentage
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Slippage;
    ///
    /// let slippage = Slippage::percent(0.5)?;
    /// assert_eq!(slippage.as_percent(), 0.5);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn as_percent(&self) -> f64 {
        self.0
    }

    /// Get slippage value as basis points
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::Slippage;
    ///
    /// let slippage = Slippage::percent(0.5)?;
    /// assert_eq!(slippage.as_bps(), 50);
    ///
    /// let slippage = Slippage::bps(100)?;
    /// assert_eq!(slippage.as_bps(), 100);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn as_bps(&self) -> u16 {
        (self.0 * 100.0).round() as u16
    }

    /// Common slippage values for convenience
    pub const ZERO: Result<Self, &'static str> = Ok(Self(0.0));

    /// 0.1% slippage (10 basis points)
    pub fn low() -> Self {
        Self(0.1)
    }

    /// 0.5% slippage (50 basis points) - recommended for most swaps
    pub fn standard() -> Self {
        Self(0.5)
    }

    /// 1% slippage (100 basis points)
    pub fn medium() -> Self {
        Self(1.0)
    }

    /// 3% slippage (300 basis points) - high slippage for volatile pairs
    pub fn high() -> Self {
        Self(3.0)
    }
}

impl fmt::Display for Slippage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}%", self.0)
    }
}

impl From<Slippage> for f64 {
    fn from(slippage: Slippage) -> Self {
        slippage.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_constructor() {
        let slippage = Slippage::percent(0.5).unwrap();
        assert_eq!(slippage.as_percent(), 0.5);

        let slippage = Slippage::percent(1.0).unwrap();
        assert_eq!(slippage.as_percent(), 1.0);

        let slippage = Slippage::percent(100.0).unwrap();
        assert_eq!(slippage.as_percent(), 100.0);

        // Edge case: 0%
        let slippage = Slippage::percent(0.0).unwrap();
        assert_eq!(slippage.as_percent(), 0.0);
    }

    #[test]
    fn test_percent_validation() {
        // Too high
        assert!(Slippage::percent(100.1).is_err());
        assert!(Slippage::percent(150.0).is_err());

        // Negative
        assert!(Slippage::percent(-0.1).is_err());
        assert!(Slippage::percent(-50.0).is_err());
    }

    #[test]
    fn test_bps_constructor() {
        let slippage = Slippage::bps(50).unwrap();
        assert_eq!(slippage.as_bps(), 50);
        assert_eq!(slippage.as_percent(), 0.5);

        let slippage = Slippage::bps(100).unwrap();
        assert_eq!(slippage.as_bps(), 100);
        assert_eq!(slippage.as_percent(), 1.0);

        let slippage = Slippage::bps(10000).unwrap();
        assert_eq!(slippage.as_bps(), 10000);
        assert_eq!(slippage.as_percent(), 100.0);

        // Edge case: 0 bps
        let slippage = Slippage::bps(0).unwrap();
        assert_eq!(slippage.as_bps(), 0);
        assert_eq!(slippage.as_percent(), 0.0);
    }

    #[test]
    fn test_bps_validation() {
        // Too high
        assert!(Slippage::bps(10001).is_err());
        assert!(Slippage::bps(15000).is_err());
        assert!(Slippage::bps(u16::MAX).is_err());
    }

    #[test]
    fn test_convenience_methods() {
        assert_eq!(Slippage::low().as_percent(), 0.1);
        assert_eq!(Slippage::standard().as_percent(), 0.5);
        assert_eq!(Slippage::medium().as_percent(), 1.0);
        assert_eq!(Slippage::high().as_percent(), 3.0);
    }

    #[test]
    fn test_display() {
        let slippage = Slippage::percent(0.5).unwrap();
        assert_eq!(format!("{slippage}"), "0.50%");

        let slippage = Slippage::bps(100).unwrap();
        assert_eq!(format!("{slippage}"), "1.00%");

        let slippage = Slippage::high();
        assert_eq!(format!("{slippage}"), "3.00%");
    }

    #[test]
    fn test_conversions() {
        let slippage = Slippage::percent(0.5).unwrap();

        // as_percent
        assert_eq!(slippage.as_percent(), 0.5);

        // as_bps
        assert_eq!(slippage.as_bps(), 50);

        // Into f64
        let value: f64 = slippage.into();
        assert_eq!(value, 0.5);
    }

    #[test]
    fn test_serialization() {
        let slippage = Slippage::percent(0.5).unwrap();

        // Serialize
        let json = serde_json::to_string(&slippage).unwrap();
        assert_eq!(json, "0.5");

        // Deserialize
        let deserialized: Slippage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.as_percent(), 0.5);
    }

    #[test]
    fn test_equality_and_ordering() {
        let s1 = Slippage::percent(0.5).unwrap();
        let s2 = Slippage::bps(50).unwrap();
        let s3 = Slippage::percent(1.0).unwrap();

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
        assert!(s1 < s3);
        assert!(s3 > s1);
    }
}
