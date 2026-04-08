use reqwest::Method;

use crate::client::Client;
use crate::error::VynFiError;
use crate::types::*;

/// Usage resource — credit balance and daily usage breakdown.
pub struct Usage<'a> {
    client: &'a Client,
}

impl<'a> Usage<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get the current credit usage summary (balance, totals, burn rate).
    pub async fn summary(&self) -> Result<UsageSummary, VynFiError> {
        self.client.request(Method::GET, "/v1/usage").await
    }

    /// Get daily credit usage. Optionally specify the number of days to look
    /// back (defaults to server-side default, typically 30).
    pub async fn daily(&self, days: Option<u32>) -> Result<DailyUsageResponse, VynFiError> {
        match days {
            Some(d) => {
                let params = [("days", d.to_string())];
                self.client
                    .request_with_params(Method::GET, "/v1/usage/daily", &params)
                    .await
            }
            None => self.client.request(Method::GET, "/v1/usage/daily").await,
        }
    }
}
