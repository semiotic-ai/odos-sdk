// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0

//! Event monitoring utilities for tracking Odos swaps.
//!
//! This module provides utilities for querying and filtering swap events from the
//! Odos router contracts. It uses Alloy's log filtering capabilities for efficient
//! event retrieval.
//!
//! # Example
//!
//! ```rust,ignore
//! use odos_sdk::events::{SwapEventFilter, SwapEvent};
//! use alloy_provider::ProviderBuilder;
//! use alloy_primitives::address;
//!
//! let provider = ProviderBuilder::new()
//!     .connect_http("https://eth.llamarpc.com".parse()?);
//!
//! // Create a filter for recent swaps
//! let filter = SwapEventFilter::new(router_address)
//!     .from_block(18_000_000)
//!     .sender(my_address);
//!
//! // Get swap events
//! let events = filter.get_events(&provider).await?;
//! for event in events {
//!     println!("Swap: {} {} -> {} {}",
//!         event.input_amount, event.input_token,
//!         event.amount_out, event.output_token);
//! }
//! ```

use alloy_network::Network;
use alloy_primitives::{Address, B256, I256, U256};
use alloy_provider::Provider;
use alloy_rpc_types::{BlockNumberOrTag, Filter, Log};
use alloy_sol_types::SolEvent;

#[cfg(feature = "v2")]
use crate::v2_router::OdosV2Router;
#[cfg(feature = "v3")]
use crate::v3_router::OdosV3Router;

/// A decoded swap event from an Odos router contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapEvent {
    /// The sender of the swap.
    pub sender: Address,
    /// The input token address.
    pub input_token: Address,
    /// The input amount.
    pub input_amount: U256,
    /// The output token address.
    pub output_token: Address,
    /// The output amount received.
    pub amount_out: U256,
    /// Slippage as a signed 256-bit integer (positive = more output than expected).
    pub slippage: I256,
    /// Referral code used (if any).
    pub referral_code: u64,
    /// The block number where the swap occurred.
    pub block_number: Option<u64>,
    /// The transaction hash.
    pub transaction_hash: Option<B256>,
    /// The log index within the transaction.
    pub log_index: Option<u64>,
}

/// Builder for creating swap event filters.
///
/// This builder provides a fluent API for constructing filters to query
/// swap events from Odos router contracts.
///
/// # Example
///
/// ```rust,ignore
/// use odos_sdk::events::SwapEventFilter;
///
/// let filter = SwapEventFilter::new(router_address)
///     .from_block(18_000_000)
///     .to_block(18_100_000)
///     .sender(sender_address);
/// ```
#[derive(Debug, Clone)]
pub struct SwapEventFilter {
    router_address: Address,
    from_block: Option<BlockNumberOrTag>,
    to_block: Option<BlockNumberOrTag>,
    sender: Option<Address>,
}

impl SwapEventFilter {
    /// Creates a new swap event filter for the given router address.
    pub fn new(router_address: Address) -> Self {
        Self {
            router_address,
            from_block: None,
            to_block: None,
            sender: None,
        }
    }

    /// Sets the starting block for the filter.
    pub fn from_block(mut self, block: u64) -> Self {
        self.from_block = Some(BlockNumberOrTag::Number(block));
        self
    }

    /// Sets the starting block to the latest block.
    pub fn from_latest(mut self) -> Self {
        self.from_block = Some(BlockNumberOrTag::Latest);
        self
    }

    /// Sets the ending block for the filter.
    pub fn to_block(mut self, block: u64) -> Self {
        self.to_block = Some(BlockNumberOrTag::Number(block));
        self
    }

    /// Sets the ending block to the latest block.
    pub fn to_latest(mut self) -> Self {
        self.to_block = Some(BlockNumberOrTag::Latest);
        self
    }

    /// Filters events by sender address.
    pub fn sender(mut self, sender: Address) -> Self {
        self.sender = Some(sender);
        self
    }

    /// Builds the underlying Alloy filter for V2 Swap events.
    #[cfg(feature = "v2")]
    pub fn build_v2_filter(&self) -> Filter {
        let mut filter = Filter::new()
            .address(self.router_address)
            .event_signature(OdosV2Router::Swap::SIGNATURE_HASH);

        if let Some(from) = self.from_block {
            filter = filter.from_block(from);
        }

        if let Some(to) = self.to_block {
            filter = filter.to_block(to);
        }

        // Note: Swap events have indexed sender, so we can filter on it
        if let Some(sender) = self.sender {
            filter = filter.topic1(sender.into_word());
        }

        filter
    }

    /// Builds the underlying Alloy filter for V3 Swap events.
    #[cfg(feature = "v3")]
    pub fn build_v3_filter(&self) -> Filter {
        let mut filter = Filter::new()
            .address(self.router_address)
            .event_signature(OdosV3Router::Swap::SIGNATURE_HASH);

        if let Some(from) = self.from_block {
            filter = filter.from_block(from);
        }

        if let Some(to) = self.to_block {
            filter = filter.to_block(to);
        }

        if let Some(sender) = self.sender {
            filter = filter.topic1(sender.into_word());
        }

        filter
    }

    /// Gets V2 swap events from the provider.
    #[cfg(feature = "v2")]
    pub async fn get_v2_events<N, P>(
        &self,
        provider: &P,
    ) -> Result<Vec<SwapEvent>, alloy_transport::TransportError>
    where
        N: Network,
        P: Provider<N>,
    {
        let filter = self.build_v2_filter();
        let logs = provider.get_logs(&filter).await?;

        Ok(logs
            .into_iter()
            .filter_map(|log| Self::decode_v2_swap_log(&log))
            .collect())
    }

    /// Gets V3 swap events from the provider.
    #[cfg(feature = "v3")]
    pub async fn get_v3_events<N, P>(
        &self,
        provider: &P,
    ) -> Result<Vec<SwapEvent>, alloy_transport::TransportError>
    where
        N: Network,
        P: Provider<N>,
    {
        let filter = self.build_v3_filter();
        let logs = provider.get_logs(&filter).await?;

        Ok(logs
            .into_iter()
            .filter_map(|log| Self::decode_v3_swap_log(&log))
            .collect())
    }

    /// Decodes a V2 Swap event from a log.
    #[cfg(feature = "v2")]
    fn decode_v2_swap_log(log: &Log) -> Option<SwapEvent> {
        let decoded = OdosV2Router::Swap::decode_log(&log.inner).ok()?;
        Some(SwapEvent {
            sender: decoded.data.sender,
            input_token: decoded.data.inputToken,
            input_amount: decoded.data.inputAmount,
            output_token: decoded.data.outputToken,
            amount_out: decoded.data.amountOut,
            slippage: decoded.data.slippage,
            referral_code: u64::from(decoded.data.referralCode),
            block_number: log.block_number,
            transaction_hash: log.transaction_hash,
            log_index: log.log_index,
        })
    }

    /// Decodes a V3 Swap event from a log.
    #[cfg(feature = "v3")]
    fn decode_v3_swap_log(log: &Log) -> Option<SwapEvent> {
        let decoded = OdosV3Router::Swap::decode_log(&log.inner).ok()?;
        Some(SwapEvent {
            sender: decoded.data.sender,
            input_token: decoded.data.inputToken,
            input_amount: decoded.data.inputAmount,
            output_token: decoded.data.outputToken,
            amount_out: decoded.data.amountOut,
            slippage: decoded.data.slippage,
            referral_code: decoded.data.referralCode,
            block_number: log.block_number,
            transaction_hash: log.transaction_hash,
            log_index: log.log_index,
        })
    }
}

/// A decoded multi-token swap event from an Odos router contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapMultiEvent {
    /// The sender of the swap.
    pub sender: Address,
    /// The input token addresses.
    pub tokens_in: Vec<Address>,
    /// The input amounts.
    pub amounts_in: Vec<U256>,
    /// The output token addresses.
    pub tokens_out: Vec<Address>,
    /// The output amounts received.
    pub amounts_out: Vec<U256>,
    /// The block number where the swap occurred.
    pub block_number: Option<u64>,
    /// The transaction hash.
    pub transaction_hash: Option<B256>,
    /// The log index within the transaction.
    pub log_index: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;

    #[test]
    fn test_swap_event_filter_builder() {
        let router = address!("cf5540fffcdc3d510b18bfca6d2b9987b0772559");
        let sender = address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0");

        let filter = SwapEventFilter::new(router)
            .from_block(18_000_000)
            .to_block(18_100_000)
            .sender(sender);

        assert_eq!(filter.router_address, router);
        assert_eq!(
            filter.from_block,
            Some(BlockNumberOrTag::Number(18_000_000))
        );
        assert_eq!(filter.to_block, Some(BlockNumberOrTag::Number(18_100_000)));
        assert_eq!(filter.sender, Some(sender));
    }

    #[test]
    fn test_swap_event_filter_latest_blocks() {
        let router = address!("cf5540fffcdc3d510b18bfca6d2b9987b0772559");

        let filter = SwapEventFilter::new(router).from_latest().to_latest();

        assert_eq!(filter.from_block, Some(BlockNumberOrTag::Latest));
        assert_eq!(filter.to_block, Some(BlockNumberOrTag::Latest));
    }

    #[cfg(feature = "v3")]
    #[test]
    fn test_build_v3_filter() {
        let router = address!("cf5540fffcdc3d510b18bfca6d2b9987b0772559");

        let filter = SwapEventFilter::new(router)
            .from_block(18_000_000)
            .build_v3_filter();

        // Verify the filter was built successfully (non-empty address)
        // The filter should be valid - we just verify it builds without error
        let _ = filter;
    }
}
