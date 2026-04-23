use reqwest::Method;
use serde::Deserialize;

use crate::client::Client;
use crate::error::VynFiError;
use crate::types::*;

/// Template packs resource (VynFi API 4.1+, DS 3.2+, Team+).
pub struct TemplatePacks<'a> {
    client: &'a Client,
}

#[derive(Debug, Deserialize, Default)]
struct StringList {
    #[serde(default)]
    data: Vec<String>,
}

impl<'a> TemplatePacks<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> Result<TemplatePackList, VynFiError> {
        self.client.request(Method::GET, "/v1/template-packs").await
    }

    pub async fn create(&self, req: &CreatePackRequest) -> Result<TemplatePack, VynFiError> {
        self.client
            .request_with_body(Method::POST, "/v1/template-packs", Some(req))
            .await
    }

    pub async fn categories(&self) -> Result<Vec<String>, VynFiError> {
        // Endpoint returns a bare JSON array; deserialise to Value first then
        // coerce to Vec<String>.
        let v: serde_json::Value = self
            .client
            .request(Method::GET, "/v1/template-packs/categories")
            .await?;
        Ok(match v {
            serde_json::Value::Array(xs) => xs
                .into_iter()
                .filter_map(|x| x.as_str().map(|s| s.to_owned()))
                .collect(),
            serde_json::Value::Object(map) => {
                let parsed: StringList =
                    serde_json::from_value(serde_json::Value::Object(map)).unwrap_or_default();
                parsed.data
            }
            _ => Vec::new(),
        })
    }

    pub async fn get(&self, pack_id: &str) -> Result<TemplatePack, VynFiError> {
        self.client
            .request(Method::GET, &format!("/v1/template-packs/{pack_id}"))
            .await
    }

    pub async fn update(
        &self,
        pack_id: &str,
        req: &UpdatePackRequest,
    ) -> Result<TemplatePack, VynFiError> {
        self.client
            .request_with_body(
                Method::PUT,
                &format!("/v1/template-packs/{pack_id}"),
                Some(req),
            )
            .await
    }

    pub async fn delete(&self, pack_id: &str) -> Result<DeletedResponse, VynFiError> {
        self.client
            .request(Method::DELETE, &format!("/v1/template-packs/{pack_id}"))
            .await
    }

    pub async fn validate(&self, pack_id: &str) -> Result<TemplatePackValidation, VynFiError> {
        self.client
            .request(
                Method::POST,
                &format!("/v1/template-packs/{pack_id}/validate"),
            )
            .await
    }

    pub async fn get_category(
        &self,
        pack_id: &str,
        category: &str,
    ) -> Result<TemplatePackCategoryContent, VynFiError> {
        self.client
            .request(
                Method::GET,
                &format!("/v1/template-packs/{pack_id}/categories/{category}"),
            )
            .await
    }

    pub async fn upsert_category(
        &self,
        pack_id: &str,
        category: &str,
        content_yaml: &str,
    ) -> Result<TemplatePackCategorySummary, VynFiError> {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            content_yaml: &'a str,
        }
        self.client
            .request_with_body(
                Method::PUT,
                &format!("/v1/template-packs/{pack_id}/categories/{category}"),
                Some(&Body { content_yaml }),
            )
            .await
    }

    pub async fn delete_category(
        &self,
        pack_id: &str,
        category: &str,
    ) -> Result<DeletedResponse, VynFiError> {
        self.client
            .request(
                Method::DELETE,
                &format!("/v1/template-packs/{pack_id}/categories/{category}"),
            )
            .await
    }

    pub async fn enrich_category(
        &self,
        pack_id: &str,
        req: &EnrichCategoryRequest,
    ) -> Result<TemplatePackEnrichResponse, VynFiError> {
        self.client
            .request_with_body(
                Method::POST,
                &format!("/v1/template-packs/{pack_id}/enrich"),
                Some(req),
            )
            .await
    }
}
