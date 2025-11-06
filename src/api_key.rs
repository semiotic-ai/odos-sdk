use std::{fmt, str::FromStr};

use uuid::Uuid;

use crate::{OdosError, Result};

/// API key for authenticating with the Odos API
///
/// Wraps a UUID-formatted API key in a type-safe manner with secure debug formatting
/// that redacts the key value to prevent accidental logging.
///
/// # Examples
///
/// ```rust
/// use odos_sdk::ApiKey;
/// use std::str::FromStr;
///
/// let api_key = ApiKey::from_str("35359429-2a05-4456-a48b-eef91f618d9e").unwrap();
///
/// // The key is redacted in debug output
/// println!("{:?}", api_key); // Prints: ApiKey([REDACTED])
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ApiKey(Uuid);

impl ApiKey {
    /// Create a new API key from a UUID
    ///
    /// # Arguments
    ///
    /// * `uuid` - The API key as a UUID
    ///
    /// # Examples
    ///
    /// ```rust
    /// use odos_sdk::ApiKey;
    /// use uuid::Uuid;
    /// use std::str::FromStr;
    ///
    /// let uuid = Uuid::from_str("35359429-2a05-4456-a48b-eef91f618d9e").unwrap();
    /// let api_key = ApiKey::new(uuid);
    /// ```
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    ///
    /// # Security
    ///
    /// Be careful when using this method - avoid logging or displaying
    /// the raw key value in production.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Get the API key as a string
    ///
    /// # Security
    ///
    /// Be careful when using this method - avoid logging or displaying
    /// the raw key value in production.
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for ApiKey {
    type Err = OdosError;

    fn from_str(s: &str) -> Result<Self> {
        let uuid = Uuid::from_str(s).map_err(|e| {
            OdosError::invalid_input(format!("Invalid API key format (expected UUID): {}", e))
        })?;
        Ok(Self(uuid))
    }
}

impl From<Uuid> for ApiKey {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Secure Debug implementation that redacts the API key
impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey([REDACTED])")
    }
}

/// Display implementation that redacts the API key
impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_new() {
        let uuid = Uuid::new_v4();
        let api_key = ApiKey::new(uuid);
        assert_eq!(api_key.as_uuid(), &uuid);
    }

    #[test]
    fn test_api_key_from_uuid() {
        let uuid = Uuid::new_v4();
        let api_key = ApiKey::from(uuid);
        assert_eq!(api_key.as_uuid(), &uuid);
    }

    #[test]
    fn test_api_key_from_str() {
        let uuid = Uuid::new_v4();
        let key = uuid.to_string();
        let api_key = ApiKey::from_str(&key).unwrap();
        assert_eq!(api_key.as_str(), key);
    }

    #[test]
    fn test_api_key_from_str_invalid() {
        let result = ApiKey::from_str("not-a-uuid");
        assert!(result.is_err());
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Invalid API key format"));
        }
    }

    #[test]
    fn test_api_key_debug_redacted() {
        let uuid = Uuid::new_v4();
        let api_key = ApiKey::new(uuid);
        let debug_output = format!("{:?}", api_key);
        assert_eq!(debug_output, "ApiKey([REDACTED])");
        let uuid_str = uuid.to_string();
        assert!(!debug_output.contains(&uuid_str));
    }

    #[test]
    fn test_api_key_display_redacted() {
        let uuid = Uuid::new_v4();
        let api_key = ApiKey::new(uuid);
        let display_output = format!("{}", api_key);
        assert_eq!(display_output, "[REDACTED]");
        let uuid_str = uuid.to_string();
        assert!(!display_output.contains(&uuid_str));
    }

    #[test]
    fn test_api_key_copy() {
        let uuid = Uuid::new_v4();
        let api_key1 = ApiKey::new(uuid);
        let api_key2 = api_key1; // Copy, not clone
        assert_eq!(api_key1.as_str(), api_key2.as_str());
    }

    #[test]
    fn test_api_key_equality() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();

        let api_key1 = ApiKey::new(uuid1);
        let api_key2 = ApiKey::new(uuid1);
        let api_key3 = ApiKey::new(uuid2);

        assert_eq!(api_key1, api_key2);
        assert_ne!(api_key1, api_key3);
    }
}
