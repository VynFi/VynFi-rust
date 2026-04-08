use reqwest::Method;

use crate::client::{extract_list, Client};
use crate::error::VynFiError;
use crate::types::*;

/// Webhooks resource — create, list, update, delete, and test webhooks.
pub struct Webhooks<'a> {
    client: &'a Client,
}

impl<'a> Webhooks<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create a new webhook. The response includes the signing secret — store
    /// it securely, as it cannot be retrieved again.
    pub async fn create(&self, req: &CreateWebhookRequest) -> Result<WebhookCreated, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/webhooks", Some(req))
            .await
    }

    /// List all webhooks for the current account.
    pub async fn list(&self) -> Result<Vec<Webhook>, VynFiError> {
        let value: serde_json::Value = self.client.request(Method::GET, "/v1/webhooks").await?;
        extract_list(value)
    }

    /// Get a single webhook by ID, including recent delivery history.
    pub async fn get(&self, webhook_id: &str) -> Result<WebhookDetail, VynFiError> {
        self.client
            .request(Method::GET, &format!("/v1/webhooks/{}", webhook_id))
            .await
    }

    /// Update a webhook's URL, events, or status.
    pub async fn update(
        &self,
        webhook_id: &str,
        req: &UpdateWebhookRequest,
    ) -> Result<Webhook, VynFiError> {
        self.client
            .request_with_body(
                Method::PATCH,
                &format!("/v1/webhooks/{}", webhook_id),
                Some(req),
            )
            .await
    }

    /// Delete a webhook. This action is irreversible.
    pub async fn delete(&self, webhook_id: &str) -> Result<(), VynFiError> {
        self.client
            .request::<()>(Method::DELETE, &format!("/v1/webhooks/{}", webhook_id))
            .await
    }

    /// Send a test event to the webhook.
    pub async fn test(&self, webhook_id: &str) -> Result<serde_json::Value, VynFiError> {
        self.client
            .request_with_body::<serde_json::Value, ()>(
                Method::POST,
                &format!("/v1/webhooks/{}/test", webhook_id),
                None,
            )
            .await
    }
}
