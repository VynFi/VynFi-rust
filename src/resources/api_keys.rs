use reqwest::Method;

use crate::client::{extract_list, Client};
use crate::error::VynFiError;
use crate::types::*;

/// API-key management resource — create, list, update, and revoke API keys.
pub struct ApiKeys<'a> {
    client: &'a Client,
}

impl<'a> ApiKeys<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create a new API key. The response includes the full secret — store it
    /// securely, as it cannot be retrieved again.
    pub async fn create(&self, req: &CreateApiKeyRequest) -> Result<ApiKeyCreated, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/api-keys", Some(req))
            .await
    }

    /// List all API keys for the current account.
    pub async fn list(&self) -> Result<Vec<ApiKey>, VynFiError> {
        let value: serde_json::Value = self.client.request(Method::GET, "/v1/api-keys").await?;
        extract_list(value)
    }

    /// Get a single API key by ID.
    pub async fn get(&self, key_id: &str) -> Result<ApiKey, VynFiError> {
        self.client
            .request(Method::GET, &format!("/v1/api-keys/{}", key_id))
            .await
    }

    /// Update an API key's name or scopes.
    pub async fn update(
        &self,
        key_id: &str,
        req: &UpdateApiKeyRequest,
    ) -> Result<ApiKey, VynFiError> {
        self.client
            .request_with_body(
                Method::PATCH,
                &format!("/v1/api-keys/{}", key_id),
                Some(req),
            )
            .await
    }

    /// Revoke (delete) an API key. This action is irreversible.
    pub async fn revoke(&self, key_id: &str) -> Result<(), VynFiError> {
        self.client
            .request_raw(Method::DELETE, &format!("/v1/api-keys/{}", key_id))
            .await?;
        Ok(())
    }
}
