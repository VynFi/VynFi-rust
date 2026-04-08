//! Blocking (synchronous) VynFi client.
//!
//! Requires the `blocking` feature flag.
//!
//! ```no_run
//! use vynfi::blocking::Client;
//! let client = Client::builder("vf_live_...").build().unwrap();
//! let sectors = client.catalog().list_sectors().unwrap();
//! ```

use std::time::Duration;

use crate::error::VynFiError;
use crate::types::*;

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Synchronous VynFi client.
///
/// Wraps the async [`crate::Client`] with an internal single-threaded Tokio
/// runtime, following the same pattern as `reqwest::blocking::Client`.
///
/// **Important:** Do *not* use this client from within an async context — the
/// internal `block_on` call will panic if a Tokio runtime is already running on
/// the current thread.
pub struct Client {
    inner: crate::Client,
    rt: tokio::runtime::Runtime,
}

impl Client {
    /// Returns a [`ClientBuilder`] that accepts an API key.
    pub fn builder(api_key: impl Into<String>) -> ClientBuilder {
        ClientBuilder {
            api_key: api_key.into(),
            base_url: None,
            timeout: None,
            max_retries: None,
        }
    }

    // -- Resource accessors ---------------------------------------------------

    pub fn jobs(&self) -> Jobs<'_> {
        Jobs { client: self }
    }

    pub fn catalog(&self) -> Catalog<'_> {
        Catalog { client: self }
    }

    pub fn usage(&self) -> Usage<'_> {
        Usage { client: self }
    }

    pub fn api_keys(&self) -> ApiKeys<'_> {
        ApiKeys { client: self }
    }

    pub fn quality(&self) -> Quality<'_> {
        Quality { client: self }
    }

    pub fn webhooks(&self) -> Webhooks<'_> {
        Webhooks { client: self }
    }

    pub fn billing(&self) -> Billing<'_> {
        Billing { client: self }
    }

    fn block_on<F: std::future::Future>(&self, f: F) -> F::Output {
        self.rt.block_on(f)
    }
}

// ---------------------------------------------------------------------------
// ClientBuilder
// ---------------------------------------------------------------------------

pub struct ClientBuilder {
    api_key: String,
    base_url: Option<String>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
}

impl ClientBuilder {
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    pub fn build(self) -> Result<Client, VynFiError> {
        let mut builder = crate::Client::builder(&self.api_key);
        if let Some(url) = self.base_url {
            builder = builder.base_url(url);
        }
        if let Some(t) = self.timeout {
            builder = builder.timeout(t);
        }
        if let Some(r) = self.max_retries {
            builder = builder.max_retries(r);
        }
        let inner = builder.build()?;

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| VynFiError::Config(e.to_string()))?;

        Ok(Client { inner, rt })
    }
}

// ---------------------------------------------------------------------------
// Jobs
// ---------------------------------------------------------------------------

pub struct Jobs<'a> {
    client: &'a Client,
}

impl Jobs<'_> {
    pub fn generate(&self, req: &GenerateRequest) -> Result<SubmitJobResponse, VynFiError> {
        self.client.block_on(self.client.inner.jobs().generate(req))
    }

    pub fn generate_config(
        &self,
        req: &GenerateConfigRequest,
    ) -> Result<SubmitJobResponse, VynFiError> {
        self.client
            .block_on(self.client.inner.jobs().generate_config(req))
    }

    pub fn generate_quick(&self, req: &GenerateRequest) -> Result<QuickJobResponse, VynFiError> {
        self.client
            .block_on(self.client.inner.jobs().generate_quick(req))
    }

    pub fn list(&self, params: &crate::ListJobsParams) -> Result<JobList, VynFiError> {
        self.client.block_on(self.client.inner.jobs().list(params))
    }

    pub fn get(&self, job_id: &str) -> Result<Job, VynFiError> {
        self.client.block_on(self.client.inner.jobs().get(job_id))
    }

    pub fn cancel(&self, job_id: &str) -> Result<CancelJobResponse, VynFiError> {
        self.client
            .block_on(self.client.inner.jobs().cancel(job_id))
    }

    pub fn download(&self, job_id: &str) -> Result<bytes::Bytes, VynFiError> {
        self.client
            .block_on(self.client.inner.jobs().download(job_id))
    }

    pub fn download_file(&self, job_id: &str, file: &str) -> Result<bytes::Bytes, VynFiError> {
        self.client
            .block_on(self.client.inner.jobs().download_file(job_id, file))
    }

    // Note: no stream() — SSE streaming does not make sense in blocking mode.
}

// ---------------------------------------------------------------------------
// Catalog
// ---------------------------------------------------------------------------

pub struct Catalog<'a> {
    client: &'a Client,
}

impl Catalog<'_> {
    pub fn list_sectors(&self) -> Result<Vec<SectorSummary>, VynFiError> {
        self.client
            .block_on(self.client.inner.catalog().list_sectors())
    }

    pub fn get_sector(&self, slug: &str) -> Result<Sector, VynFiError> {
        self.client
            .block_on(self.client.inner.catalog().get_sector(slug))
    }

    pub fn list(
        &self,
        sector: Option<&str>,
        search: Option<&str>,
    ) -> Result<Vec<CatalogItem>, VynFiError> {
        self.client
            .block_on(self.client.inner.catalog().list(sector, search))
    }

    pub fn get_fingerprint(&self, sector: &str, profile: &str) -> Result<Fingerprint, VynFiError> {
        self.client
            .block_on(self.client.inner.catalog().get_fingerprint(sector, profile))
    }
}

// ---------------------------------------------------------------------------
// Usage
// ---------------------------------------------------------------------------

pub struct Usage<'a> {
    client: &'a Client,
}

impl Usage<'_> {
    pub fn summary(&self, days: Option<i32>) -> Result<UsageSummary, VynFiError> {
        self.client
            .block_on(self.client.inner.usage().summary(days))
    }

    pub fn daily(&self, days: Option<i32>) -> Result<DailyUsageResponse, VynFiError> {
        self.client.block_on(self.client.inner.usage().daily(days))
    }
}

// ---------------------------------------------------------------------------
// ApiKeys
// ---------------------------------------------------------------------------

pub struct ApiKeys<'a> {
    client: &'a Client,
}

impl ApiKeys<'_> {
    pub fn create(&self, req: &CreateApiKeyRequest) -> Result<ApiKeyCreated, VynFiError> {
        self.client
            .block_on(self.client.inner.api_keys().create(req))
    }

    pub fn list(&self) -> Result<Vec<ApiKey>, VynFiError> {
        self.client.block_on(self.client.inner.api_keys().list())
    }

    pub fn get(&self, key_id: &str) -> Result<ApiKey, VynFiError> {
        self.client
            .block_on(self.client.inner.api_keys().get(key_id))
    }

    pub fn update(&self, key_id: &str, req: &UpdateApiKeyRequest) -> Result<ApiKey, VynFiError> {
        self.client
            .block_on(self.client.inner.api_keys().update(key_id, req))
    }

    pub fn revoke(&self, key_id: &str) -> Result<RevokeKeyResponse, VynFiError> {
        self.client
            .block_on(self.client.inner.api_keys().revoke(key_id))
    }
}

// ---------------------------------------------------------------------------
// Quality
// ---------------------------------------------------------------------------

pub struct Quality<'a> {
    client: &'a Client,
}

impl Quality<'_> {
    pub fn scores(&self) -> Result<Vec<QualityScore>, VynFiError> {
        self.client.block_on(self.client.inner.quality().scores())
    }

    pub fn timeline(&self, days: Option<i64>) -> Result<Vec<DailyQuality>, VynFiError> {
        self.client
            .block_on(self.client.inner.quality().timeline(days))
    }
}

// ---------------------------------------------------------------------------
// Webhooks
// ---------------------------------------------------------------------------

pub struct Webhooks<'a> {
    client: &'a Client,
}

impl Webhooks<'_> {
    pub fn create(&self, req: &CreateWebhookRequest) -> Result<WebhookCreated, VynFiError> {
        self.client
            .block_on(self.client.inner.webhooks().create(req))
    }

    pub fn list(&self) -> Result<Vec<Webhook>, VynFiError> {
        self.client.block_on(self.client.inner.webhooks().list())
    }

    pub fn get(&self, webhook_id: &str) -> Result<WebhookDetail, VynFiError> {
        self.client
            .block_on(self.client.inner.webhooks().get(webhook_id))
    }

    pub fn update(
        &self,
        webhook_id: &str,
        req: &UpdateWebhookRequest,
    ) -> Result<Webhook, VynFiError> {
        self.client
            .block_on(self.client.inner.webhooks().update(webhook_id, req))
    }

    pub fn delete(&self, webhook_id: &str) -> Result<(), VynFiError> {
        self.client
            .block_on(self.client.inner.webhooks().delete(webhook_id))
    }

    pub fn test(&self, webhook_id: &str) -> Result<serde_json::Value, VynFiError> {
        self.client
            .block_on(self.client.inner.webhooks().test(webhook_id))
    }
}

// ---------------------------------------------------------------------------
// Billing
// ---------------------------------------------------------------------------

pub struct Billing<'a> {
    client: &'a Client,
}

impl Billing<'_> {
    pub fn subscription(&self) -> Result<Subscription, VynFiError> {
        self.client
            .block_on(self.client.inner.billing().subscription())
    }

    pub fn checkout(&self, req: &CheckoutRequest) -> Result<CheckoutResponse, VynFiError> {
        self.client
            .block_on(self.client.inner.billing().checkout(req))
    }

    pub fn portal(&self) -> Result<PortalResponse, VynFiError> {
        self.client.block_on(self.client.inner.billing().portal())
    }

    pub fn invoices(&self) -> Result<Vec<Invoice>, VynFiError> {
        self.client.block_on(self.client.inner.billing().invoices())
    }

    pub fn payment_method(&self) -> Result<serde_json::Value, VynFiError> {
        self.client
            .block_on(self.client.inner.billing().payment_method())
    }
}
