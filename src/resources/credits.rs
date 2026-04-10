use reqwest::Method;

use crate::client::Client;
use crate::error::VynFiError;
use crate::types::*;

/// Credits resource — purchase prepaid credits and check balances.
pub struct Credits<'a> {
    client: &'a Client,
}

impl<'a> Credits<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Purchase a prepaid credit pack. Returns a Stripe checkout URL.
    ///
    /// Valid packs: `"10k"`, `"50k"`, `"100k"`, `"500k"`, `"1m"`.
    pub async fn purchase(
        &self,
        req: &PurchaseCreditsRequest,
    ) -> Result<PurchaseCreditsResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/credits/purchase", Some(req))
            .await
    }

    /// Get the current prepaid credit balance and active batches.
    pub async fn balance(&self) -> Result<PrepaidBalanceResponse, VynFiError> {
        self.client
            .request(Method::GET, "/v1/credits/balance")
            .await
    }

    /// Get the full prepaid credit history (including expired batches).
    pub async fn history(&self) -> Result<PrepaidHistoryResponse, VynFiError> {
        self.client
            .request(Method::GET, "/v1/credits/history")
            .await
    }
}
