use reqwest::Method;

use crate::client::{extract_list, Client};
use crate::error::VynFiError;
use crate::types::*;

/// Billing resource — subscription details, invoices, and payment methods.
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

    /// List all invoices for the current account.
    pub async fn invoices(&self) -> Result<Vec<Invoice>, VynFiError> {
        let value: serde_json::Value = self
            .client
            .request(Method::GET, "/v1/billing/invoices")
            .await?;
        extract_list(value)
    }

    /// Get the payment method on file.
    pub async fn payment_method(&self) -> Result<PaymentMethod, VynFiError> {
        self.client
            .request(Method::GET, "/v1/billing/payment-method")
            .await
    }
}
