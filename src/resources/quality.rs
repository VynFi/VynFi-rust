use reqwest::Method;

use crate::client::{extract_list, Client};
use crate::error::VynFiError;
use crate::types::*;

/// Quality metrics resource — retrieve quality scores and trend data.
pub struct Quality<'a> {
    client: &'a Client,
}

impl<'a> Quality<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get quality scores for recent generation jobs.
    pub async fn scores(&self) -> Result<Vec<QualityScore>, VynFiError> {
        let value: serde_json::Value = self
            .client
            .request(Method::GET, "/v1/quality/scores")
            .await?;
        extract_list(value)
    }

    /// Get daily aggregate quality scores over time. Optionally specify the
    /// number of days to look back (default 30, max 365).
    pub async fn timeline(&self, days: Option<i64>) -> Result<Vec<DailyQuality>, VynFiError> {
        let value: serde_json::Value = match days {
            Some(d) => {
                let params = [("days", d.to_string())];
                self.client
                    .request_with_params(Method::GET, "/v1/quality/timeline", &params)
                    .await?
            }
            None => {
                self.client
                    .request(Method::GET, "/v1/quality/timeline")
                    .await?
            }
        };
        extract_list(value)
    }
}
