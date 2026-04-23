use reqwest::Method;

use crate::client::Client;
use crate::error::VynFiError;
use crate::types::*;

/// AI co-pilot resource (DS 3.0+, Scale+ tier).
pub struct Ai<'a> {
    client: &'a Client,
}

impl<'a> Ai<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Free-text chat with the VynFi dashboard co-pilot.
    pub async fn chat(&self, req: &AiChatRequest) -> Result<AiChatResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/ai/chat", Some(req))
            .await
    }
}
