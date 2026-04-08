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
        let value: serde_json::Value = self.client.request(Method::GET, "/v1/sectors").await?;
        extract_list(value)
    }

    /// Get a single sector by slug, including its full table definitions.
    pub async fn get_sector(&self, slug: &str) -> Result<Sector, VynFiError> {
        self.client
            .request(Method::GET, &format!("/v1/sectors/{}", slug))
            .await
    }

    /// List catalog items, optionally filtered by sector or search term.
    pub async fn list(
        &self,
        sector: Option<&str>,
        search: Option<&str>,
    ) -> Result<Vec<CatalogItem>, VynFiError> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(s) = sector {
            params.push(("sector", s.to_string()));
        }
        if let Some(q) = search {
            params.push(("search", q.to_string()));
        }

        let value: serde_json::Value = if params.is_empty() {
            self.client.request(Method::GET, "/v1/catalog").await?
        } else {
            self.client
                .request_with_params(Method::GET, "/v1/catalog", &params)
                .await?
        };
        extract_list(value)
    }

    /// Get a fingerprint profile for a specific sector and profile slug.
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
