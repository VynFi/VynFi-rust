use reqwest::Method;

use crate::client::{extract_list, Client};
use crate::error::VynFiError;
use crate::types::*;

/// Parameters for listing saved configs.
#[derive(Debug, Default)]
pub struct ListConfigsParams {
    /// Maximum number of configs to return.
    pub limit: Option<i64>,
    /// Offset for pagination.
    pub offset: Option<i64>,
}

/// Configs resource — save, list, update, validate, and estimate generation
/// configs.
pub struct Configs<'a> {
    client: &'a Client,
}

impl<'a> Configs<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Save a new generation config.
    pub async fn create(&self, req: &CreateConfigRequest) -> Result<SavedConfig, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/configs", Some(req))
            .await
    }

    /// List saved configs with optional pagination.
    pub async fn list(&self, params: &ListConfigsParams) -> Result<Vec<SavedConfig>, VynFiError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(offset) = params.offset {
            query.push(("offset", offset.to_string()));
        }

        let value: serde_json::Value = self
            .client
            .request_with_params(Method::GET, "/v1/configs", &query)
            .await?;
        extract_list(value)
    }

    /// Get a saved config by ID.
    pub async fn get(&self, config_id: &str) -> Result<SavedConfig, VynFiError> {
        self.client
            .request(Method::GET, &format!("/v1/configs/{}", config_id))
            .await
    }

    /// Update a saved config.
    pub async fn update(
        &self,
        config_id: &str,
        req: &UpdateConfigRequest,
    ) -> Result<SavedConfig, VynFiError> {
        self.client
            .request_with_body(
                Method::PUT,
                &format!("/v1/configs/{}", config_id),
                Some(req),
            )
            .await
    }

    /// Delete a saved config.
    pub async fn delete(&self, config_id: &str) -> Result<DeletedResponse, VynFiError> {
        self.client
            .request(Method::DELETE, &format!("/v1/configs/{}", config_id))
            .await
    }

    /// Validate a generation config. Returns errors and warnings.
    pub async fn validate(
        &self,
        req: &ValidateConfigRequest,
    ) -> Result<ValidateConfigResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/config/validate", Some(req))
            .await
    }

    /// Estimate the credit cost of a generation config.
    pub async fn estimate_cost(
        &self,
        req: &EstimateCostRequest,
    ) -> Result<EstimateCostResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/config/estimate-cost", Some(req))
            .await
    }

    /// Compose a config from multiple layers.
    pub async fn compose(
        &self,
        req: &ComposeConfigRequest,
    ) -> Result<ComposeConfigResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/config/compose", Some(req))
            .await
    }

    /// Estimate the total archive size of a generation config (DS 3.0+).
    pub async fn estimate_size(
        &self,
        req: &EstimateSizeRequest,
    ) -> Result<EstimateSizeResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/configs/estimate-size", Some(req))
            .await
    }

    /// Submit a raw DataSynth YAML config for validation (Scale+).
    pub async fn submit_raw(
        &self,
        req: &RawConfigRequest,
    ) -> Result<RawConfigResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/configs/raw", Some(req))
            .await
    }

    /// Build a config from a natural-language description (Scale+).
    pub async fn from_description(
        &self,
        description: impl Into<String>,
    ) -> Result<NlConfigResponse, VynFiError> {
        let req = NlDescriptionRequest {
            description: description.into(),
        };
        self.client
            .request_with_body(Method::POST, "/v1/configs/from-description", Some(&req))
            .await
    }

    /// Build a config from a Swiss VynCo company profile (Scale+).
    ///
    /// Supply either `uid` or `name` on the request.
    pub async fn from_company(
        &self,
        req: &NlCompanyRequest,
    ) -> Result<CompanyConfigResponse, VynFiError> {
        if req.uid.is_none() && req.name.is_none() {
            return Err(VynFiError::Config(
                "from_company() requires either uid or name".into(),
            ));
        }
        self.client
            .request_with_body(Method::POST, "/v1/configs/from-company", Some(req))
            .await
    }
}
