use reqwest::Method;

use crate::client::Client;
use crate::error::VynFiError;
use crate::types::*;

/// Adversarial probing resource (DS 3.0+, Enterprise tier).
pub struct Adversarial<'a> {
    client: &'a Client,
}

impl<'a> Adversarial<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn probe(
        &self,
        body: &serde_json::Value,
    ) -> Result<AdversarialProbeResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/adversarial/probe", Some(body))
            .await
    }

    pub async fn results(&self, probe_id: &str) -> Result<AdversarialProbeResults, VynFiError> {
        self.client
            .request(
                Method::GET,
                &format!("/v1/adversarial/{probe_id}/results"),
            )
            .await
    }
}
