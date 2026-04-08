use reqwest::Method;

use crate::client::{extract_list, Client};
use crate::error::VynFiError;
use crate::types::*;

/// Billing resource — subscription, checkout, invoices, and payment methods.
pub struct Billing<'a> {
    client: &'a Client,
}

impl<'a> Billing<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get the current subscription details (tier, status, period end).
    pub async fn subscription(&self) -> Result<Subscription, VynFiError> {
        self.client
            .request(Method::GET, "/v1/billing/subscription")
            .await
    }

    /// Create a Stripe checkout session to subscribe to a plan.
    pub async fn checkout(&self, req: &CheckoutRequest) -> Result<CheckoutResponse, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/billing/checkout", Some(req))
            .await
    }

    /// Create a Stripe billing portal session for managing payment methods
    /// and invoices.
    pub async fn portal(&self) -> Result<PortalResponse, VynFiError> {
        self.client
            .request_with_body::<PortalResponse, ()>(Method::POST, "/v1/billing/portal", None)
            .await
    }

    /// List all invoices for the current account.
    pub async fn invoices(&self) -> Result<Vec<Invoice>, VynFiError> {
        let value: serde_json::Value = self
            .client
            .request(Method::GET, "/v1/billing/invoices")
            .await?;
        extract_list(value)
    }

    /// Get the payment method on file.
    pub async fn payment_method(&self) -> Result<serde_json::Value, VynFiError> {
        self.client
            .request(Method::GET, "/v1/billing/payment-method")
            .await
    }
}
