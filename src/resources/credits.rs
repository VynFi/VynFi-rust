use reqwest::Method;

use crate::client::Client;
use crate::error::VynFiError;
use crate::types::*;

/// Credits resource — purchase credit packs and view prepaid balance.
pub struct Credits<'a> {
    client: &'a Client,
}

impl<'a> Credits<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Purchase a credit pack. Returns a Stripe checkout URL.
    ///
    /// Valid pack names: `"starter"`, `"builder"`, `"pro"`.
    pub async fn purchase(
        &self,
        req: &PurchaseCreditsRequest,
    ) -> Result<PurchaseCreditsResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/credits/purchase", Some(req))
            .await
    }

    /// Get the current prepaid credit balance and active batches.
    pub async fn balance(&self) -> Result<CreditBalance, VynFiError> {
        self.client
            .request(Method::GET, "/v1/credits/balance")
            .await
    }

    /// Get full credit purchase history with batch details.
    pub async fn history(&self) -> Result<CreditHistory, VynFiError> {
        self.client
            .request(Method::GET, "/v1/credits/history")
            .await
    }
}
