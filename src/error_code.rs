// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! Strongly-typed Odos API error codes
//!
//! This module provides type-safe representations of error codes returned by the Odos API.
//! Error codes are organized into categories (General, Algo, Internal Service, Validation)
//! and provide helper methods for error inspection and retryability logic.
//!
//! # [Error Code Index](https://docs.odos.xyz/build/api_errors)
//!
//! - **General API Errors (1XXX)**: Basic API errors
//! - **Algo/Quote Errors (2XXX)**: Routing and quote generation errors
//! - **Internal Service Errors (3XXX)**: Backend service errors
//! - **Validation Errors (4XXX)**: Request validation errors
//! - **Internal Errors (5XXX)**: System-level internal errors

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Strongly-typed trace ID for Odos API error tracking
///
/// Wraps a UUID to prevent confusion with other UUID types in the system.
/// Each error response from Odos includes a unique trace ID for debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TraceId(pub Uuid);

impl TraceId {
    /// Create a new TraceId from a UUID
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for TraceId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Error code category for grouping related errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// General API errors (1XXX)
    General,
    /// Algorithm/Quote errors (2XXX)
    Algo,
    /// Internal service errors (3XXX)
    InternalService,
    /// Validation errors (4XXX)
    Validation,
    /// System internal errors (5XXX)
    Internal,
    /// Unknown error code
    Unknown,
}

/// Strongly-typed Odos API error codes
///
/// Each variant represents a specific error condition documented by Odos.
/// Error codes are grouped by category (1XXX-5XXX ranges).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OdosErrorCode {
    // General Odos API errors (1XXX)
    /// General API error (1000)
    ApiError,

    // Odos Algo/Quote errors (2XXX)
    /// No viable routing path found (2000)
    NoViablePath,
    /// Algorithm validation error (2400)
    AlgoValidationError,
    /// Algorithm connection error (2997)
    AlgoConnectionError,
    /// Algorithm timeout (2998)
    AlgoTimeout,
    /// Algorithm internal error (2999)
    AlgoInternal,

    // Odos Internal Service errors (3XXX)
    /// Internal service error (3000)
    InternalServiceError,

    // Config service errors (31XX)
    /// Config service internal error (3100)
    ConfigInternal,
    /// Config service connection error (3101)
    ConfigConnectionError,
    /// Config service timeout (3102)
    ConfigTimeout,

    // Transaction assembly errors (311X)
    /// Transaction assembly internal error (3110)
    TxnAssemblyInternal,
    /// Transaction assembly connection error (3111)
    TxnAssemblyConnectionError,
    /// Transaction assembly timeout (3112)
    TxnAssemblyTimeout,

    // Chain data errors (312X)
    /// Chain data internal error (3120)
    ChainDataInternal,
    /// Chain data connection error (3121)
    ChainDataConnectionError,
    /// Chain data timeout (3122)
    ChainDataTimeout,

    // Pricing service errors (313X)
    /// Pricing service internal error (3130)
    PricingInternal,
    /// Pricing service connection error (3131)
    PricingConnectionError,
    /// Pricing service timeout (3132)
    PricingTimeout,

    // Gas service errors (314X)
    /// Gas service internal error (3140)
    GasInternal,
    /// Gas service connection error (3141)
    GasConnectionError,
    /// Gas service timeout (3142)
    GasTimeout,
    /// Gas data unavailable (3143)
    GasUnavailable,

    // Odos Validation errors (4XXX)
    /// Invalid request (4000)
    InvalidRequest,

    // General/Quote errors (40XX)
    /// Invalid chain ID (4001)
    InvalidChainId,
    /// Invalid input tokens (4002)
    InvalidInputTokens,
    /// Invalid output tokens (4003)
    InvalidOutputTokens,
    /// Invalid user address (4004)
    InvalidUserAddr,
    /// Blocked user address (4005)
    BlockedUserAddr,
    /// Slippage tolerance too high (4006)
    TooSlippery,
    /// Same token for input and output (4007)
    SameInputOutput,
    /// Multiple zap outputs not supported (4008)
    MultiZapOutput,
    /// Invalid token count (4009)
    InvalidTokenCount,
    /// Invalid token address (4010)
    InvalidTokenAddr,
    /// Non-integer token amount (4011)
    NonIntegerTokenAmount,
    /// Negative token amount (4012)
    NegativeTokenAmount,
    /// Same tokens in input and output (4013)
    SameInputOutputTokens,
    /// Token is blacklisted (4014)
    TokenBlacklisted,
    /// Invalid token proportions (4015)
    InvalidTokenProportions,
    /// Token routing unavailable (4016)
    TokenRoutingUnavailable,
    /// Invalid referral code (4017)
    InvalidReferralCode,
    /// Invalid token amount (4018)
    InvalidTokenAmount,
    /// Non-string token amount (4019)
    NonStringTokenAmount,

    // Assembly errors (41XX)
    /// Invalid assembly request (4100)
    InvalidAssemblyRequest,
    /// Invalid user address in assembly (4101)
    InvalidAssemblyUserAddr,
    /// Invalid receiver address (4102)
    InvalidReceiverAddr,

    // Swap errors (42XX)
    /// Invalid swap request (4200)
    InvalidSwapRequest,
    /// User address required (4201)
    UserAddrRequired,

    // Odos Internal errors (5XXX)
    /// Internal error (5000)
    InternalError,
    /// Swap unavailable (5001)
    SwapUnavailable,
    /// Price check failure (5002)
    PriceCheckFailure,
    /// Default gas calculation failure (5003)
    DefaultGasFailure,

    /// Unknown error code
    Unknown(u16),
}

impl OdosErrorCode {
    /// Get the numeric error code value
    pub fn code(&self) -> u16 {
        match self {
            Self::ApiError => 1000,
            Self::NoViablePath => 2000,
            Self::AlgoValidationError => 2400,
            Self::AlgoConnectionError => 2997,
            Self::AlgoTimeout => 2998,
            Self::AlgoInternal => 2999,
            Self::InternalServiceError => 3000,
            Self::ConfigInternal => 3100,
            Self::ConfigConnectionError => 3101,
            Self::ConfigTimeout => 3102,
            Self::TxnAssemblyInternal => 3110,
            Self::TxnAssemblyConnectionError => 3111,
            Self::TxnAssemblyTimeout => 3112,
            Self::ChainDataInternal => 3120,
            Self::ChainDataConnectionError => 3121,
            Self::ChainDataTimeout => 3122,
            Self::PricingInternal => 3130,
            Self::PricingConnectionError => 3131,
            Self::PricingTimeout => 3132,
            Self::GasInternal => 3140,
            Self::GasConnectionError => 3141,
            Self::GasTimeout => 3142,
            Self::GasUnavailable => 3143,
            Self::InvalidRequest => 4000,
            Self::InvalidChainId => 4001,
            Self::InvalidInputTokens => 4002,
            Self::InvalidOutputTokens => 4003,
            Self::InvalidUserAddr => 4004,
            Self::BlockedUserAddr => 4005,
            Self::TooSlippery => 4006,
            Self::SameInputOutput => 4007,
            Self::MultiZapOutput => 4008,
            Self::InvalidTokenCount => 4009,
            Self::InvalidTokenAddr => 4010,
            Self::NonIntegerTokenAmount => 4011,
            Self::NegativeTokenAmount => 4012,
            Self::SameInputOutputTokens => 4013,
            Self::TokenBlacklisted => 4014,
            Self::InvalidTokenProportions => 4015,
            Self::TokenRoutingUnavailable => 4016,
            Self::InvalidReferralCode => 4017,
            Self::InvalidTokenAmount => 4018,
            Self::NonStringTokenAmount => 4019,
            Self::InvalidAssemblyRequest => 4100,
            Self::InvalidAssemblyUserAddr => 4101,
            Self::InvalidReceiverAddr => 4102,
            Self::InvalidSwapRequest => 4200,
            Self::UserAddrRequired => 4201,
            Self::InternalError => 5000,
            Self::SwapUnavailable => 5001,
            Self::PriceCheckFailure => 5002,
            Self::DefaultGasFailure => 5003,
            Self::Unknown(code) => *code,
        }
    }

    /// Get the error category
    pub fn category(&self) -> ErrorCategory {
        let code = self.code();
        match code {
            1000..=1999 => ErrorCategory::General,
            2000..=2999 => ErrorCategory::Algo,
            3000..=3999 => ErrorCategory::InternalService,
            4000..=4999 => ErrorCategory::Validation,
            5000..=5999 => ErrorCategory::Internal,
            _ => ErrorCategory::Unknown,
        }
    }

    /// Check if this is a general API error
    pub fn is_general_error(&self) -> bool {
        matches!(self.category(), ErrorCategory::General)
    }

    /// Check if this is an algorithm/quote error
    pub fn is_algo_error(&self) -> bool {
        matches!(self.category(), ErrorCategory::Algo)
    }

    /// Check if this is an internal service error
    pub fn is_internal_service_error(&self) -> bool {
        matches!(self.category(), ErrorCategory::InternalService)
    }

    /// Check if this is a validation error
    pub fn is_validation_error(&self) -> bool {
        matches!(self.category(), ErrorCategory::Validation)
    }

    /// Check if this is an internal error
    pub fn is_internal_error(&self) -> bool {
        matches!(self.category(), ErrorCategory::Internal)
    }

    /// Check if this specific error is no viable path
    pub fn is_no_viable_path(&self) -> bool {
        matches!(self, Self::NoViablePath)
    }

    /// Check if this is an invalid chain ID error
    pub fn is_invalid_chain_id(&self) -> bool {
        matches!(self, Self::InvalidChainId)
    }

    /// Check if this is a blocked user address error
    pub fn is_blocked_user(&self) -> bool {
        matches!(self, Self::BlockedUserAddr)
    }

    /// Check if this is a timeout error (any service)
    pub fn is_timeout(&self) -> bool {
        matches!(
            self,
            Self::AlgoTimeout
                | Self::ConfigTimeout
                | Self::TxnAssemblyTimeout
                | Self::ChainDataTimeout
                | Self::PricingTimeout
                | Self::GasTimeout
        )
    }

    /// Check if this is a connection error (any service)
    pub fn is_connection_error(&self) -> bool {
        matches!(
            self,
            Self::AlgoConnectionError
                | Self::ConfigConnectionError
                | Self::TxnAssemblyConnectionError
                | Self::ChainDataConnectionError
                | Self::PricingConnectionError
                | Self::GasConnectionError
        )
    }

    /// Check if this error indicates the request should be retried
    ///
    /// Retryable errors include:
    /// - Timeouts (algo, config, assembly, chain data, pricing, gas)
    /// - Connection errors (all services)
    /// - Internal errors (algo, services, general)
    /// - Gas unavailable
    pub fn is_retryable(&self) -> bool {
        self.is_timeout()
            || self.is_connection_error()
            || matches!(
                self,
                Self::AlgoInternal
                    | Self::ConfigInternal
                    | Self::TxnAssemblyInternal
                    | Self::ChainDataInternal
                    | Self::PricingInternal
                    | Self::GasInternal
                    | Self::GasUnavailable
                    | Self::InternalServiceError
                    | Self::InternalError
            )
    }

    /// Check if this error indicates the token cannot be routed
    ///
    /// This is NOT an error condition - it's Odos correctly responding that
    /// the token cannot be swapped. Common reasons include:
    /// - Token is not supported by Odos (TokenRoutingUnavailable)
    /// - Token is blacklisted (TokenBlacklisted)
    /// - Token is not recognized (InvalidInputTokens/InvalidOutputTokens)
    /// - No liquidity path exists (NoViablePath)
    ///
    /// These should be tracked separately from actual errors for metrics purposes.
    pub fn is_unroutable_token(&self) -> bool {
        matches!(
            self,
            Self::TokenRoutingUnavailable
                | Self::TokenBlacklisted
                | Self::InvalidInputTokens
                | Self::InvalidOutputTokens
                | Self::NoViablePath
        )
    }
}

impl From<u16> for OdosErrorCode {
    fn from(code: u16) -> Self {
        match code {
            1000 => Self::ApiError,
            2000 => Self::NoViablePath,
            2400 => Self::AlgoValidationError,
            2997 => Self::AlgoConnectionError,
            2998 => Self::AlgoTimeout,
            2999 => Self::AlgoInternal,
            3000 => Self::InternalServiceError,
            3100 => Self::ConfigInternal,
            3101 => Self::ConfigConnectionError,
            3102 => Self::ConfigTimeout,
            3110 => Self::TxnAssemblyInternal,
            3111 => Self::TxnAssemblyConnectionError,
            3112 => Self::TxnAssemblyTimeout,
            3120 => Self::ChainDataInternal,
            3121 => Self::ChainDataConnectionError,
            3122 => Self::ChainDataTimeout,
            3130 => Self::PricingInternal,
            3131 => Self::PricingConnectionError,
            3132 => Self::PricingTimeout,
            3140 => Self::GasInternal,
            3141 => Self::GasConnectionError,
            3142 => Self::GasTimeout,
            3143 => Self::GasUnavailable,
            4000 => Self::InvalidRequest,
            4001 => Self::InvalidChainId,
            4002 => Self::InvalidInputTokens,
            4003 => Self::InvalidOutputTokens,
            4004 => Self::InvalidUserAddr,
            4005 => Self::BlockedUserAddr,
            4006 => Self::TooSlippery,
            4007 => Self::SameInputOutput,
            4008 => Self::MultiZapOutput,
            4009 => Self::InvalidTokenCount,
            4010 => Self::InvalidTokenAddr,
            4011 => Self::NonIntegerTokenAmount,
            4012 => Self::NegativeTokenAmount,
            4013 => Self::SameInputOutputTokens,
            4014 => Self::TokenBlacklisted,
            4015 => Self::InvalidTokenProportions,
            4016 => Self::TokenRoutingUnavailable,
            4017 => Self::InvalidReferralCode,
            4018 => Self::InvalidTokenAmount,
            4019 => Self::NonStringTokenAmount,
            4100 => Self::InvalidAssemblyRequest,
            4101 => Self::InvalidAssemblyUserAddr,
            4102 => Self::InvalidReceiverAddr,
            4200 => Self::InvalidSwapRequest,
            4201 => Self::UserAddrRequired,
            5000 => Self::InternalError,
            5001 => Self::SwapUnavailable,
            5002 => Self::PriceCheckFailure,
            5003 => Self::DefaultGasFailure,
            _ => Self::Unknown(code),
        }
    }
}

impl From<OdosErrorCode> for u16 {
    fn from(error_code: OdosErrorCode) -> Self {
        error_code.code()
    }
}

impl fmt::Display for OdosErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ApiError => write!(f, "1000 (API_ERROR)"),
            Self::NoViablePath => write!(f, "2000 (NO_VIABLE_PATH)"),
            Self::AlgoValidationError => write!(f, "2400 (ALGO_VALIDATION_ERR)"),
            Self::AlgoConnectionError => write!(f, "2997 (ALGO_CONN_ERR)"),
            Self::AlgoTimeout => write!(f, "2998 (ALGO_TIMEOUT)"),
            Self::AlgoInternal => write!(f, "2999 (ALGO_INTERNAL)"),
            Self::InternalServiceError => write!(f, "3000 (INTERNAL_SERVICE_ERROR)"),
            Self::ConfigInternal => write!(f, "3100 (CONFIG_INTERNAL)"),
            Self::ConfigConnectionError => write!(f, "3101 (CONFIG_CONN_ERR)"),
            Self::ConfigTimeout => write!(f, "3102 (CONFIG_TIMEOUT)"),
            Self::TxnAssemblyInternal => write!(f, "3110 (TXN_ASSEMBLY_INTERNAL)"),
            Self::TxnAssemblyConnectionError => write!(f, "3111 (TXN_ASSEMBLY_CONN_ERR)"),
            Self::TxnAssemblyTimeout => write!(f, "3112 (TXN_ASSEMBLY_TIMEOUT)"),
            Self::ChainDataInternal => write!(f, "3120 (CHAIN_DATA_INTERNAL)"),
            Self::ChainDataConnectionError => write!(f, "3121 (CHAIN_DATA_CONN_ERR)"),
            Self::ChainDataTimeout => write!(f, "3122 (CHAIN_DATA_TIMEOUT)"),
            Self::PricingInternal => write!(f, "3130 (PRICING_INTERNAL)"),
            Self::PricingConnectionError => write!(f, "3131 (PRICING_CONN_ERR)"),
            Self::PricingTimeout => write!(f, "3132 (PRICING_TIMEOUT)"),
            Self::GasInternal => write!(f, "3140 (GAS_INTERNAL)"),
            Self::GasConnectionError => write!(f, "3141 (GAS_CONN_ERR)"),
            Self::GasTimeout => write!(f, "3142 (GAS_TIMEOUT)"),
            Self::GasUnavailable => write!(f, "3143 (GAS_UNAVAILABLE)"),
            Self::InvalidRequest => write!(f, "4000 (INVALID_REQUEST)"),
            Self::InvalidChainId => write!(f, "4001 (INVALID_CHAIN_ID)"),
            Self::InvalidInputTokens => write!(f, "4002 (INVALID_INPUT_TOKENS)"),
            Self::InvalidOutputTokens => write!(f, "4003 (INVALID_OUTPUT_TOKENS)"),
            Self::InvalidUserAddr => write!(f, "4004 (INVALID_USER_ADDR)"),
            Self::BlockedUserAddr => write!(f, "4005 (BLOCKED_USER_ADDR)"),
            Self::TooSlippery => write!(f, "4006 (TOO_SLIPPERY)"),
            Self::SameInputOutput => write!(f, "4007 (SAME_INPUT_OUTPUT)"),
            Self::MultiZapOutput => write!(f, "4008 (MULTI_ZAP_OUTPUT)"),
            Self::InvalidTokenCount => write!(f, "4009 (INVALID_TOKEN_COUNT)"),
            Self::InvalidTokenAddr => write!(f, "4010 (INVALID_TOKEN_ADDR)"),
            Self::NonIntegerTokenAmount => write!(f, "4011 (NON_INTEGER_TOKEN_AMOUNT)"),
            Self::NegativeTokenAmount => write!(f, "4012 (NEGATIVE_TOKEN_AMOUNT)"),
            Self::SameInputOutputTokens => write!(f, "4013 (SAME_INPUT_OUTPUT_TOKENS)"),
            Self::TokenBlacklisted => write!(f, "4014 (TOKEN_BLACKLISTED)"),
            Self::InvalidTokenProportions => write!(f, "4015 (INVALID_TOKEN_PROPORTIONS)"),
            Self::TokenRoutingUnavailable => write!(f, "4016 (TOKEN_ROUTING_UNAVAILABLE)"),
            Self::InvalidReferralCode => write!(f, "4017 (INVALID_REFERRAL_CODE)"),
            Self::InvalidTokenAmount => write!(f, "4018 (INVALID_TOKEN_AMOUNT)"),
            Self::NonStringTokenAmount => write!(f, "4019 (NON_STRING_TOKEN_AMOUNT)"),
            Self::InvalidAssemblyRequest => write!(f, "4100 (INVALID_ASSEMBLY_REQUEST)"),
            Self::InvalidAssemblyUserAddr => write!(f, "4101 (INVALID_USER_ADDR)"),
            Self::InvalidReceiverAddr => write!(f, "4102 (INVALID_RECEIVER_ADDR)"),
            Self::InvalidSwapRequest => write!(f, "4200 (INVALID_SWAP_REQUEST)"),
            Self::UserAddrRequired => write!(f, "4201 (USER_ADDR_REQ)"),
            Self::InternalError => write!(f, "5000 (INTERNAL_ERROR)"),
            Self::SwapUnavailable => write!(f, "5001 (SWAP_UNAVAILABLE)"),
            Self::PriceCheckFailure => write!(f, "5002 (PRICE_CHECK_FAILURE)"),
            Self::DefaultGasFailure => write!(f, "5003 (DEFAULT_GAS_FAILURE)"),
            Self::Unknown(code) => write!(f, "{code} (UNKNOWN)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_id_creation() {
        let uuid = Uuid::parse_str("10becdc8-a021-4491-8201-a17b657204e0").unwrap();
        let trace_id = TraceId::new(uuid);
        assert_eq!(trace_id.as_uuid(), uuid);
        assert_eq!(trace_id.to_string(), "10becdc8-a021-4491-8201-a17b657204e0");
    }

    #[test]
    fn test_error_code_from_u16() {
        assert_eq!(OdosErrorCode::from(2999), OdosErrorCode::AlgoInternal);
        assert_eq!(OdosErrorCode::from(4001), OdosErrorCode::InvalidChainId);
        assert_eq!(OdosErrorCode::from(3142), OdosErrorCode::GasTimeout);
        assert_eq!(OdosErrorCode::from(9999), OdosErrorCode::Unknown(9999));
    }

    #[test]
    fn test_error_code_to_u16() {
        assert_eq!(OdosErrorCode::AlgoInternal.code(), 2999);
        assert_eq!(OdosErrorCode::InvalidChainId.code(), 4001);
        assert_eq!(OdosErrorCode::Unknown(9999).code(), 9999);
    }

    #[test]
    fn test_error_categories() {
        assert!(OdosErrorCode::ApiError.is_general_error());
        assert!(OdosErrorCode::NoViablePath.is_algo_error());
        assert!(OdosErrorCode::ConfigInternal.is_internal_service_error());
        assert!(OdosErrorCode::InvalidChainId.is_validation_error());
        assert!(OdosErrorCode::InternalError.is_internal_error());
    }

    #[test]
    fn test_specific_error_checks() {
        assert!(OdosErrorCode::NoViablePath.is_no_viable_path());
        assert!(OdosErrorCode::InvalidChainId.is_invalid_chain_id());
        assert!(OdosErrorCode::BlockedUserAddr.is_blocked_user());

        assert!(!OdosErrorCode::ApiError.is_no_viable_path());
        assert!(!OdosErrorCode::AlgoInternal.is_invalid_chain_id());
    }

    #[test]
    fn test_timeout_detection() {
        assert!(OdosErrorCode::AlgoTimeout.is_timeout());
        assert!(OdosErrorCode::ConfigTimeout.is_timeout());
        assert!(OdosErrorCode::TxnAssemblyTimeout.is_timeout());
        assert!(OdosErrorCode::ChainDataTimeout.is_timeout());
        assert!(OdosErrorCode::PricingTimeout.is_timeout());
        assert!(OdosErrorCode::GasTimeout.is_timeout());

        assert!(!OdosErrorCode::AlgoInternal.is_timeout());
        assert!(!OdosErrorCode::InvalidChainId.is_timeout());
    }

    #[test]
    fn test_connection_error_detection() {
        assert!(OdosErrorCode::AlgoConnectionError.is_connection_error());
        assert!(OdosErrorCode::ConfigConnectionError.is_connection_error());
        assert!(OdosErrorCode::GasConnectionError.is_connection_error());

        assert!(!OdosErrorCode::AlgoInternal.is_connection_error());
        assert!(!OdosErrorCode::InvalidChainId.is_connection_error());
    }

    #[test]
    fn test_retryability() {
        // Timeouts are retryable
        assert!(OdosErrorCode::AlgoTimeout.is_retryable());
        assert!(OdosErrorCode::GasTimeout.is_retryable());

        // Connection errors are retryable
        assert!(OdosErrorCode::AlgoConnectionError.is_retryable());
        assert!(OdosErrorCode::PricingConnectionError.is_retryable());

        // Internal errors are retryable
        assert!(OdosErrorCode::AlgoInternal.is_retryable());
        assert!(OdosErrorCode::InternalServiceError.is_retryable());
        assert!(OdosErrorCode::GasUnavailable.is_retryable());

        // Validation errors are not retryable
        assert!(!OdosErrorCode::InvalidChainId.is_retryable());
        assert!(!OdosErrorCode::BlockedUserAddr.is_retryable());
        assert!(!OdosErrorCode::InvalidTokenAmount.is_retryable());

        // No viable path is not retryable
        assert!(!OdosErrorCode::NoViablePath.is_retryable());
    }

    #[test]
    fn test_display_format() {
        assert_eq!(
            OdosErrorCode::AlgoInternal.to_string(),
            "2999 (ALGO_INTERNAL)"
        );
        assert_eq!(
            OdosErrorCode::InvalidChainId.to_string(),
            "4001 (INVALID_CHAIN_ID)"
        );
        assert_eq!(OdosErrorCode::Unknown(9999).to_string(), "9999 (UNKNOWN)");
    }

    #[test]
    fn test_unroutable_token_detection() {
        // These indicate the token cannot be routed (expected behavior, not errors)
        assert!(OdosErrorCode::NoViablePath.is_unroutable_token());
        assert!(OdosErrorCode::TokenRoutingUnavailable.is_unroutable_token());
        assert!(OdosErrorCode::TokenBlacklisted.is_unroutable_token());
        assert!(OdosErrorCode::InvalidInputTokens.is_unroutable_token());
        assert!(OdosErrorCode::InvalidOutputTokens.is_unroutable_token());

        // These are NOT unroutable token indicators
        assert!(!OdosErrorCode::AlgoTimeout.is_unroutable_token());
        assert!(!OdosErrorCode::InternalError.is_unroutable_token());
        assert!(!OdosErrorCode::InvalidChainId.is_unroutable_token());
        assert!(!OdosErrorCode::BlockedUserAddr.is_unroutable_token());
    }
}
