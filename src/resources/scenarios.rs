use reqwest::Method;

use crate::client::Client;
use crate::error::VynFiError;
use crate::types::*;

/// Scenarios resource — create and run what-if analysis scenarios.
pub struct Scenarios<'a> {
    client: &'a Client,
}

impl<'a> Scenarios<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all scenarios.
    pub async fn list(&self) -> Result<Vec<Scenario>, VynFiError> {
        self.client.request(Method::GET, "/v1/scenarios").await
    }

    /// Create a new what-if scenario.
    pub async fn create(&self, req: &CreateScenarioRequest) -> Result<Scenario, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/scenarios", Some(req))
            .await
    }

    /// Run a scenario (creates baseline and counterfactual generation jobs).
    pub async fn run(&self, scenario_id: &str) -> Result<Scenario, VynFiError> {
        self.client
            .request_with_body::<Scenario, ()>(
                Method::POST,
                &format!("/v1/scenarios/{}/run", scenario_id),
                None,
            )
            .await
    }

    /// Get the diff analysis between baseline and counterfactual results.
    pub async fn diff(&self, scenario_id: &str) -> Result<Scenario, VynFiError> {
        self.client
            .request(Method::GET, &format!("/v1/scenarios/{}/diff", scenario_id))
            .await
    }

    /// List available scenario templates.
    pub async fn templates(&self) -> Result<Vec<ScenarioTemplate>, VynFiError> {
        self.client
            .request(Method::GET, "/v1/scenarios/templates")
            .await
    }
}
