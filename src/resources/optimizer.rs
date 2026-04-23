use reqwest::Method;

use crate::client::Client;
use crate::error::VynFiError;
use crate::types::*;

/// Audit optimizer resource (VynFi API 4.1+, DS 4.1.2+, Scale+).
///
/// Wraps the six `POST /v1/optimizer/*` endpoints. DS 4.1.2 ships stub
/// reports; deeper analytics light up in later DS 4.1.x / 4.2.x releases.
/// `OptimizerResponse.report` is opaque so this resource keeps working as
/// the upstream stub → real-analytics migration progresses.
pub struct Optimizer<'a> {
    client: &'a Client,
}

impl<'a> Optimizer<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn risk_scope(
        &self,
        req: &RiskScopeRequest,
    ) -> Result<OptimizerResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/optimizer/risk-scope", Some(req))
            .await
    }

    pub async fn portfolio(&self, req: &PortfolioRequest) -> Result<OptimizerResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/optimizer/portfolio", Some(req))
            .await
    }

    pub async fn resources(&self, req: &ResourcesRequest) -> Result<OptimizerResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/optimizer/resources", Some(req))
            .await
    }

    pub async fn conformance(
        &self,
        req: &ConformanceRequest,
    ) -> Result<OptimizerResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/optimizer/conformance", Some(req))
            .await
    }

    pub async fn monte_carlo(
        &self,
        req: &MonteCarloRequest,
    ) -> Result<OptimizerResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/optimizer/monte-carlo", Some(req))
            .await
    }

    pub async fn calibration(
        &self,
        req: &CalibrationRequest,
    ) -> Result<OptimizerResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/optimizer/calibration", Some(req))
            .await
    }
}
