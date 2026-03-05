use reqwest::Method;

use crate::client::{extract_list, Client};
use crate::error::VynFiError;
use crate::types::*;

/// Catalog resource — browse sectors, tables, and fingerprint profiles.
pub struct Catalog<'a> {
    client: &'a Client,
}

impl<'a> Catalog<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all available sectors (without full table definitions).
    pub async fn list_sectors(&self) -> Result<Vec<SectorSummary>, VynFiError> {
        let value: serde_json::Value = self
            .client
            .request(Method::GET, "/v1/catalog/sectors")
            .await?;
        extract_list(value)
    }

    /// Get a single sector by slug, including its full table definitions.
    pub async fn get_sector(&self, slug: &str) -> Result<Sector, VynFiError> {
        self.client
            .request(Method::GET, &format!("/v1/catalog/sectors/{}", slug))
            .await
    }

    /// List all catalog items across all sectors.
    pub async fn list(&self) -> Result<Vec<CatalogItem>, VynFiError> {
        let value: serde_json::Value = self.client.request(Method::GET, "/v1/catalog").await?;
        extract_list(value)
    }

    /// Get a fingerprint profile for a specific sector and profile name.
    pub async fn get_fingerprint(
        &self,
        sector: &str,
        profile: &str,
    ) -> Result<Fingerprint, VynFiError> {
        self.client
            .request(Method::GET, &format!("/v1/catalog/{}/{}", sector, profile))
            .await
    }
}
