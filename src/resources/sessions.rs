use reqwest::Method;

use crate::client::Client;
use crate::error::VynFiError;
use crate::types::*;

/// Sessions resource — manage multi-period generation sessions.
pub struct Sessions<'a> {
    client: &'a Client,
}

impl<'a> Sessions<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all generation sessions.
    pub async fn list(&self) -> Result<Vec<GenerationSession>, VynFiError> {
        self.client.request(Method::GET, "/v1/sessions").await
    }

    /// Create a new multi-period generation session.
    pub async fn create(
        &self,
        req: &CreateSessionRequest,
    ) -> Result<GenerationSession, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/sessions", Some(req))
            .await
    }

    /// Add more periods to an existing session.
    pub async fn extend(
        &self,
        session_id: &str,
        req: &ExtendSessionRequest,
    ) -> Result<GenerationSession, VynFiError> {
        self.client
            .request_with_body(
                Method::POST,
                &format!("/v1/sessions/{}/extend", session_id),
                Some(req),
            )
            .await
    }

    /// Generate the next period of a session. Returns the session state with
    /// the new job's details.
    pub async fn generate_next(
        &self,
        session_id: &str,
    ) -> Result<GenerateSessionResponse, VynFiError> {
        self.client
            .request_with_body::<GenerateSessionResponse, ()>(
                Method::POST,
                &format!("/v1/sessions/{}/generate", session_id),
                None,
            )
            .await
    }
}
