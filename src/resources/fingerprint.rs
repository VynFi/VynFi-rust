use reqwest::Method;

use crate::client::Client;
use crate::error::VynFiError;
use crate::types::*;

/// Fingerprint synthesis resource (DS 3.0+, Team+).
pub struct Fingerprint<'a> {
    client: &'a Client,
}

impl<'a> Fingerprint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Submit a `.dsf` fingerprint for privacy-preserving synthesis.
    ///
    /// The body is passed through as an arbitrary JSON request so callers
    /// can craft whatever shape the portal expects for the current
    /// synthesis backend (statistical / neural / hybrid).
    pub async fn synthesize(
        &self,
        body: &serde_json::Value,
    ) -> Result<FingerprintSynthesisResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/fingerprint/synthesize", Some(body))
            .await
    }
}
