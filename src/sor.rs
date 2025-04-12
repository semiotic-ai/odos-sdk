use alloy_network::TransactionBuilder;
use alloy_primitives::{Address, hex};
use alloy_rpc_types::TransactionRequest;
use reqwest::{Client, Response};
use serde_json::Value;
use tracing::{debug, info, instrument};

use crate::{ASSEMBLE_URL, AssembleRequest, AssemblyResponse, Swap, parse_value};

use super::TransactionData;

use crate::{QuoteRequest, SingleQuoteResponse};

#[derive(Debug, Clone, Default)]
pub struct OdosSorV2 {
    client: Client,
}

impl OdosSorV2 {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Get a swap quote using Odos API
    ///
    /// Takes a [`QuoteRequest`] and returns a [`SingleQuoteResponse`].
    #[instrument(skip(self), level = "debug")]
    pub async fn get_swap_quote(
        &self,
        quote_request: &QuoteRequest,
    ) -> anyhow::Result<SingleQuoteResponse> {
        let response = self
            .client
            .post("https://api.odos.xyz/sor/quote/v2")
            .header("accept", "application/json")
            .json(quote_request)
            .send()
            .await?;

        debug!(response = ?response);

        if response.status().is_success() {
            let single_quote_response = response.json().await?;
            Ok(single_quote_response)
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Error in Quote Response: {error_text}"))
        }
    }

    #[instrument(skip(self), level = "debug")]
    pub async fn get_assemble_response(
        &self,
        assemble_request: AssembleRequest,
    ) -> Result<Response, reqwest::Error> {
        self.client
            .post(ASSEMBLE_URL)
            .header("Content-Type", "application/json")
            .json(&assemble_request)
            .send()
            .await
    }

    /// Assemble transaction data from a quote using the Odos Assemble API.
    #[instrument(skip(self), ret(Debug))]
    pub async fn assemble_tx_data(
        &self,
        signer_address: Address,
        output_recipient: Address,
        path_id: &str,
    ) -> anyhow::Result<TransactionData> {
        let assemble_request = AssembleRequest {
            user_addr: signer_address.to_string(),
            path_id: path_id.to_string(),
            simulate: false,
            receiver: Some(output_recipient),
        };

        let response = self.get_assemble_response(assemble_request).await?;

        if !response.status().is_success() {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to get error message".to_string());

            return Err(anyhow::anyhow!("Error in Transaction Assembly: {error}"));
        }

        let value: Value = response.json().await?;

        let AssemblyResponse { transaction, .. } = serde_json::from_value(value)?;

        Ok(transaction)
    }

    /// Build a base transaction from a swap using the Odos Assemble API,
    /// leaving gas parameters to be set by the caller.
    #[instrument(skip(self), ret(Debug))]
    pub async fn build_base_transaction(&self, swap: &Swap) -> anyhow::Result<TransactionRequest> {
        let TransactionData { data, value, .. } = self
            .assemble_tx_data(
                swap.signer_address(),
                swap.output_recipient(),
                swap.path_id(),
            )
            .await?;

        info!(value = %value, "Building base transaction");

        Ok(TransactionRequest::default()
            .with_input(hex::decode(&data)?)
            .with_value(parse_value(&value))
            .with_to(swap.router_address())
            .with_from(swap.signer_address()))
    }
}
