use alloy_primitives::{Address, U256};
use alloy_rpc_types::TransactionRequest;

use crate::{
    AssemblyRequest, Chain, OdosChain, OdosClient, QuoteRequest, ReferralCode, Result,
    SingleQuoteResponse, Slippage,
};

/// High-level swap builder for common use cases
///
/// Provides an ergonomic API for building swaps without needing to understand
/// the low-level quote → assemble → build flow.
///
/// # Examples
///
/// ## Simple swap
///
/// ```rust,no_run
/// use odos_sdk::{OdosClient, Slippage, Chain};
/// use alloy_primitives::{address, U256};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = OdosClient::new()?;
///
/// let tx = client
///     .swap()
///     .chain(Chain::ethereum())
///     .from_token(address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), U256::from(1_000_000))
///     .to_token(address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"))
///     .slippage(Slippage::percent(0.5)?)
///     .signer(address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0"))
///     .build_transaction()
///     .await?;
/// # Ok(())
/// # }
/// ```
///
/// ## With custom recipient
///
/// ```rust,no_run
/// use odos_sdk::{OdosClient, Slippage, Chain};
/// use alloy_primitives::{address, U256};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = OdosClient::new()?;
///
/// let tx = client
///     .swap()
///     .chain(Chain::arbitrum())
///     .from_token(address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), U256::from(1_000_000))
///     .to_token(address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"))
///     .slippage(Slippage::bps(50)?)
///     .signer(address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0"))
///     .recipient(address!("0000000000000000000000000000000000000001"))
///     .build_transaction()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SwapBuilder<'a> {
    client: &'a OdosClient,
    chain: Option<Chain>,
    input_token: Option<Address>,
    input_amount: Option<U256>,
    output_token: Option<Address>,
    slippage: Option<Slippage>,
    signer: Option<Address>,
    recipient: Option<Address>,
    referral: ReferralCode,
    compact: bool,
    simple: bool,
    disable_rfqs: bool,
}

impl<'a> SwapBuilder<'a> {
    /// Create a new swap builder
    pub(crate) fn new(client: &'a OdosClient) -> Self {
        Self {
            client,
            chain: None,
            input_token: None,
            input_amount: None,
            output_token: None,
            slippage: None,
            signer: None,
            recipient: None,
            referral: ReferralCode::NONE,
            compact: false,
            simple: false,
            disable_rfqs: false,
        }
    }

    /// Set the blockchain to execute the swap on
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::{OdosClient, Chain};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// let builder = client.swap().chain(Chain::ethereum());
    /// # Ok(())
    /// # }
    /// ```
    pub fn chain(mut self, chain: Chain) -> Self {
        self.chain = Some(chain);
        self
    }

    /// Set the input token and amount
    ///
    /// # Arguments
    ///
    /// * `token` - Address of the token to swap from
    /// * `amount` - Amount of input token (in token's base units)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::OdosClient;
    /// use alloy_primitives::{address, U256};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// let builder = client.swap()
    ///     .input(address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), U256::from(1_000_000));
    /// # Ok(())
    /// # }
    /// ```
    pub fn input(mut self, token: Address, amount: U256) -> Self {
        self.input_token = Some(token);
        self.input_amount = Some(amount);
        self
    }

    /// Alias for `input()` - set the token to swap from
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::OdosClient;
    /// use alloy_primitives::{address, U256};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// let builder = client.swap()
    ///     .from_token(address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), U256::from(1_000_000));
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_token(self, token: Address, amount: U256) -> Self {
        self.input(token, amount)
    }

    /// Set the output token (100% of output goes to this token)
    ///
    /// # Arguments
    ///
    /// * `token` - Address of the token to swap to
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::OdosClient;
    /// use alloy_primitives::address;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// let builder = client.swap()
    ///     .output(address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn output(mut self, token: Address) -> Self {
        self.output_token = Some(token);
        self
    }

    /// Alias for `output()` - set the token to swap to
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::OdosClient;
    /// use alloy_primitives::address;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// let builder = client.swap()
    ///     .to_token(address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_token(self, token: Address) -> Self {
        self.output(token)
    }

    /// Set the slippage tolerance
    ///
    /// # Arguments
    ///
    /// * `slippage` - Maximum acceptable slippage
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::{OdosClient, Slippage};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// let builder = client.swap()
    ///     .slippage(Slippage::percent(0.5)?);  // 0.5% slippage
    /// # Ok(())
    /// # }
    /// ```
    pub fn slippage(mut self, slippage: Slippage) -> Self {
        self.slippage = Some(slippage);
        self
    }

    /// Set the address that will sign and send the transaction
    ///
    /// # Arguments
    ///
    /// * `address` - The signer's address
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::OdosClient;
    /// use alloy_primitives::address;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// let builder = client.swap()
    ///     .signer(address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn signer(mut self, address: Address) -> Self {
        self.signer = Some(address);
        self
    }

    /// Set the recipient address for output tokens
    ///
    /// If not set, defaults to the signer address.
    ///
    /// # Arguments
    ///
    /// * `address` - The recipient's address
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::OdosClient;
    /// use alloy_primitives::address;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// let builder = client.swap()
    ///     .signer(address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0"))
    ///     .recipient(address!("0000000000000000000000000000000000000001"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn recipient(mut self, address: Address) -> Self {
        self.recipient = Some(address);
        self
    }

    /// Set the referral code
    ///
    /// # Arguments
    ///
    /// * `code` - Referral code for tracking
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::{OdosClient, ReferralCode};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// let builder = client.swap()
    ///     .referral(ReferralCode::new(42));
    /// # Ok(())
    /// # }
    /// ```
    pub fn referral(mut self, code: ReferralCode) -> Self {
        self.referral = code;
        self
    }

    /// Enable compact mode
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::OdosClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// let builder = client.swap().compact(true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Enable simple mode
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::OdosClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// let builder = client.swap().simple(true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn simple(mut self, simple: bool) -> Self {
        self.simple = simple;
        self
    }

    /// Disable RFQs (Request for Quotes)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::OdosClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    /// let builder = client.swap().disable_rfqs(true);
    /// # Ok(())
    /// # }
    /// ```
    pub fn disable_rfqs(mut self, disable: bool) -> Self {
        self.disable_rfqs = disable;
        self
    }

    /// Get a quote for this swap without building the transaction
    ///
    /// This is useful if you want to inspect the quote before proceeding.
    ///
    /// # Returns
    ///
    /// Returns a `SingleQuoteResponse` with routing information, price impact, and gas estimates.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required fields are missing
    /// - The Odos API returns an error
    /// - Network issues occur
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::{OdosClient, Chain, Slippage};
    /// use alloy_primitives::{address, U256};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    ///
    /// let quote = client
    ///     .swap()
    ///     .chain(Chain::ethereum())
    ///     .from_token(address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), U256::from(1_000_000))
    ///     .to_token(address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"))
    ///     .slippage(Slippage::percent(0.5)?)
    ///     .signer(address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0"))
    ///     .quote()
    ///     .await?;
    ///
    /// println!("Expected output: {} tokens", quote.out_amount().unwrap());
    /// println!("Price impact: {}%", quote.price_impact());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn quote(&self) -> Result<SingleQuoteResponse> {
        let chain = self
            .chain
            .ok_or_else(|| crate::OdosError::missing_data("Chain is required for swap builder"))?;

        let input_token = self.input_token.ok_or_else(|| {
            crate::OdosError::missing_data("Input token is required for swap builder")
        })?;

        let input_amount = self.input_amount.ok_or_else(|| {
            crate::OdosError::missing_data("Input amount is required for swap builder")
        })?;

        let output_token = self.output_token.ok_or_else(|| {
            crate::OdosError::missing_data("Output token is required for swap builder")
        })?;

        let slippage = self.slippage.ok_or_else(|| {
            crate::OdosError::missing_data("Slippage is required for swap builder")
        })?;

        let signer = self.signer.ok_or_else(|| {
            crate::OdosError::missing_data("Signer address is required for swap builder")
        })?;

        let quote_request = QuoteRequest::builder()
            .chain_id(chain.id())
            .input_tokens(vec![(input_token, input_amount).into()])
            .output_tokens(vec![(output_token, 1).into()])
            .slippage_limit_percent(slippage.as_percent())
            .user_addr(signer.to_string())
            .compact(self.compact)
            .simple(self.simple)
            .referral_code(self.referral.code())
            .disable_rfqs(self.disable_rfqs)
            .build();

        self.client.quote(&quote_request).await
    }

    /// Build the complete transaction for this swap
    ///
    /// This method:
    /// 1. Gets a quote from the Odos API
    /// 2. Assembles the transaction data
    /// 3. Returns a `TransactionRequest` ready for signing
    ///
    /// The returned transaction still needs gas parameters set before signing.
    ///
    /// # Returns
    ///
    /// Returns a `TransactionRequest` with:
    /// - `to`: Router contract address
    /// - `from`: Signer address
    /// - `data`: Encoded swap calldata
    /// - `value`: ETH amount to send (if swapping from native token)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required fields are missing
    /// - The Odos API returns an error
    /// - Transaction assembly fails
    /// - Network issues occur
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use odos_sdk::{OdosClient, Chain, Slippage};
    /// use alloy_primitives::{address, U256};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = OdosClient::new()?;
    ///
    /// let tx = client
    ///     .swap()
    ///     .chain(Chain::ethereum())
    ///     .from_token(address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), U256::from(1_000_000))
    ///     .to_token(address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"))
    ///     .slippage(Slippage::percent(0.5)?)
    ///     .signer(address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0"))
    ///     .build_transaction()
    ///     .await?;
    ///
    /// // Set gas parameters and sign
    /// // let tx = tx.with_gas_limit(300_000);
    /// // provider.send_transaction(tx).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build_transaction(&self) -> Result<TransactionRequest> {
        // Get quote
        let quote = self.quote().await?;

        let chain = self.chain.unwrap(); // Safe: validated in quote()
        let signer = self.signer.unwrap(); // Safe: validated in quote()
        let recipient = self.recipient.unwrap_or(signer);
        let input_token = self.input_token.unwrap(); // Safe: validated in quote()
        let input_amount = self.input_amount.unwrap(); // Safe: validated in quote()

        // Get router address for this chain
        let router_address = chain.v3_router_address()?;

        // Build swap context
        let swap_context = AssemblyRequest::builder()
            .chain(chain.inner())
            .router_address(router_address)
            .signer_address(signer)
            .output_recipient(recipient)
            .token_address(input_token)
            .token_amount(input_amount)
            .path_id(quote.path_id().to_string())
            .build();

        // Build transaction
        self.client.assemble(&swap_context).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;

    #[test]
    fn test_builder_construction() {
        let client = OdosClient::new().unwrap();
        let builder = client.swap();

        assert!(builder.chain.is_none());
        assert!(builder.input_token.is_none());
        assert!(builder.output_token.is_none());
        assert_eq!(builder.referral, ReferralCode::NONE);
    }

    #[test]
    fn test_builder_chain_methods() {
        let client = OdosClient::new().unwrap();

        let builder = client
            .swap()
            .chain(Chain::ethereum())
            .from_token(
                address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
                U256::from(1_000_000),
            )
            .to_token(address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"))
            .slippage(Slippage::standard())
            .signer(address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0"));

        assert_eq!(builder.chain.unwrap(), Chain::ethereum());
        assert_eq!(
            builder.input_token.unwrap(),
            address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
        );
        assert_eq!(builder.input_amount.unwrap(), U256::from(1_000_000));
        assert_eq!(
            builder.output_token.unwrap(),
            address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2")
        );
        assert_eq!(builder.slippage.unwrap(), Slippage::standard());
        assert_eq!(
            builder.signer.unwrap(),
            address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0")
        );
    }

    #[test]
    fn test_builder_aliases() {
        let client = OdosClient::new().unwrap();

        // Test input() vs from_token()
        let builder1 = client.swap().input(
            address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
            U256::from(1000),
        );
        let builder2 = client.swap().from_token(
            address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
            U256::from(1000),
        );

        assert_eq!(builder1.input_token, builder2.input_token);
        assert_eq!(builder1.input_amount, builder2.input_amount);

        // Test output() vs to_token()
        let builder1 = client
            .swap()
            .output(address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"));
        let builder2 = client
            .swap()
            .to_token(address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"));

        assert_eq!(builder1.output_token, builder2.output_token);
    }

    #[test]
    fn test_builder_recipient_defaults_to_signer() {
        let client = OdosClient::new().unwrap();
        let signer_addr = address!("742d35Cc6634C0532925a3b8D35f3e7a5edD29c0");

        let builder = client.swap().signer(signer_addr);

        assert_eq!(builder.signer.unwrap(), signer_addr);
        assert!(builder.recipient.is_none()); // Not set, will default in build
    }
}
