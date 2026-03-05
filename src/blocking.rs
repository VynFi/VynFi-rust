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
    ///
    /// ```no_run
    /// # use vynfi::blocking::Client;
    /// let client = Client::builder("vf_live_abc123").build().unwrap();
    /// ```
    pub fn builder(api_key: impl Into<String>) -> ClientBuilder {
        ClientBuilder {
            api_key: api_key.into(),
            base_url: None,
            timeout: None,
            max_retries: None,
        }
    }

    // -- Resource accessors ---------------------------------------------------

    /// Jobs resource — submit, list, get, cancel, and download generation jobs.
    pub fn jobs(&self) -> Jobs<'_> {
        Jobs { client: self }
    }

    /// Catalog resource — list sectors, tables, and fingerprints.
    pub fn catalog(&self) -> Catalog<'_> {
        Catalog { client: self }
    }

    /// Usage resource — credit balance and daily usage breakdown.
    pub fn usage(&self) -> Usage<'_> {
        Usage { client: self }
    }

    /// API-key management resource.
    pub fn api_keys(&self) -> ApiKeys<'_> {
        ApiKeys { client: self }
    }

    /// Quality metrics resource.
    pub fn quality(&self) -> Quality<'_> {
        Quality { client: self }
    }

    /// Webhooks resource — CRUD and delivery history.
    pub fn webhooks(&self) -> Webhooks<'_> {
        Webhooks { client: self }
    }

    /// Billing resource — subscription, invoices, payment methods.
    pub fn billing(&self) -> Billing<'_> {
        Billing { client: self }
    }

    // -- Internal helper ------------------------------------------------------

    fn block_on<F: std::future::Future>(&self, f: F) -> F::Output {
        self.rt.block_on(f)
    }
}

// ---------------------------------------------------------------------------
// ClientBuilder
// ---------------------------------------------------------------------------

/// Builder for configuring and constructing a blocking [`Client`].
///
/// ```no_run
/// # use vynfi::blocking::Client;
/// # use std::time::Duration;
/// let client = Client::builder("vf_live_abc123")
///     .base_url("https://staging-api.vynfi.com")
///     .timeout(Duration::from_secs(60))
///     .max_retries(3)
///     .build()
///     .unwrap();
/// ```
pub struct ClientBuilder {
    api_key: String,
    base_url: Option<String>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
}

impl ClientBuilder {
    /// Override the base URL (trailing slashes are stripped).
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the request timeout (default: 30 s).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the maximum number of automatic retries on 429 / 5xx (default: 2).
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    /// Build the blocking [`Client`].
    ///
    /// Returns an error if the API key is empty, the underlying HTTP client
    /// cannot be constructed, or the Tokio runtime fails to initialise.
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

/// Blocking handle for the Jobs resource.
pub struct Jobs<'a> {
    client: &'a Client,
}

impl Jobs<'_> {
    /// Submit an asynchronous generation job.
    pub fn generate(&self, req: &GenerateRequest) -> Result<SubmitJobResponse, VynFiError> {
        self.client.block_on(self.client.inner.jobs().generate(req))
    }

    /// Submit a job and poll until completion, returning the finished [`Job`].
    pub fn generate_quick(&self, req: &GenerateRequest) -> Result<Job, VynFiError> {
        self.client
            .block_on(self.client.inner.jobs().generate_quick(req))
    }

    /// List jobs with optional filtering and pagination.
    pub fn list(&self, params: &crate::ListJobsParams) -> Result<JobList, VynFiError> {
        self.client.block_on(self.client.inner.jobs().list(params))
    }

    /// Get a single job by ID.
    pub fn get(&self, job_id: &str) -> Result<Job, VynFiError> {
        self.client.block_on(self.client.inner.jobs().get(job_id))
    }

    /// Cancel a running job.
    pub fn cancel(&self, job_id: &str) -> Result<Job, VynFiError> {
        self.client
            .block_on(self.client.inner.jobs().cancel(job_id))
    }

    /// Download the output of a completed job as raw bytes.
    pub fn download(&self, job_id: &str) -> Result<bytes::Bytes, VynFiError> {
        self.client
            .block_on(self.client.inner.jobs().download(job_id))
    }

    // Note: no stream() — SSE streaming does not make sense in blocking mode.
}

// ---------------------------------------------------------------------------
// Catalog
// ---------------------------------------------------------------------------

/// Blocking handle for the Catalog resource.
pub struct Catalog<'a> {
    client: &'a Client,
}

impl Catalog<'_> {
    /// List all available sectors (summary only, no tables).
    pub fn list_sectors(&self) -> Result<Vec<SectorSummary>, VynFiError> {
        self.client
            .block_on(self.client.inner.catalog().list_sectors())
    }

    /// Get full details for a sector, including its tables and columns.
    pub fn get_sector(&self, slug: &str) -> Result<Sector, VynFiError> {
        self.client
            .block_on(self.client.inner.catalog().get_sector(slug))
    }

    /// List all catalog items across every sector.
    pub fn list(&self) -> Result<Vec<CatalogItem>, VynFiError> {
        self.client.block_on(self.client.inner.catalog().list())
    }

    /// Get a fingerprint definition by sector and profile name.
    pub fn get_fingerprint(&self, sector: &str, profile: &str) -> Result<Fingerprint, VynFiError> {
        self.client
            .block_on(self.client.inner.catalog().get_fingerprint(sector, profile))
    }
}

// ---------------------------------------------------------------------------
// Usage
// ---------------------------------------------------------------------------

/// Blocking handle for the Usage resource.
pub struct Usage<'a> {
    client: &'a Client,
}

impl Usage<'_> {
    /// Get a summary of credit balance and usage statistics.
    pub fn summary(&self) -> Result<UsageSummary, VynFiError> {
        self.client.block_on(self.client.inner.usage().summary())
    }

    /// Get daily usage breakdown with per-table totals.
    pub fn daily(&self, days: Option<u32>) -> Result<DailyUsageResponse, VynFiError> {
        self.client.block_on(self.client.inner.usage().daily(days))
    }
}

// ---------------------------------------------------------------------------
// ApiKeys
// ---------------------------------------------------------------------------

/// Blocking handle for the API Keys resource.
pub struct ApiKeys<'a> {
    client: &'a Client,
}

impl ApiKeys<'_> {
    /// Create a new API key.
    pub fn create(&self, req: &CreateApiKeyRequest) -> Result<ApiKeyCreated, VynFiError> {
        self.client
            .block_on(self.client.inner.api_keys().create(req))
    }

    /// List all API keys for the authenticated user.
    pub fn list(&self) -> Result<Vec<ApiKey>, VynFiError> {
        self.client.block_on(self.client.inner.api_keys().list())
    }

    /// Get a single API key by ID.
    pub fn get(&self, key_id: &str) -> Result<ApiKey, VynFiError> {
        self.client
            .block_on(self.client.inner.api_keys().get(key_id))
    }

    /// Update an API key's name or scopes.
    pub fn update(&self, key_id: &str, req: &UpdateApiKeyRequest) -> Result<ApiKey, VynFiError> {
        self.client
            .block_on(self.client.inner.api_keys().update(key_id, req))
    }

    /// Revoke (delete) an API key.
    pub fn revoke(&self, key_id: &str) -> Result<(), VynFiError> {
        self.client
            .block_on(self.client.inner.api_keys().revoke(key_id))
    }
}

// ---------------------------------------------------------------------------
// Quality
// ---------------------------------------------------------------------------

/// Blocking handle for the Quality resource.
pub struct Quality<'a> {
    client: &'a Client,
}

impl Quality<'_> {
    /// Get quality scores for all jobs.
    pub fn scores(&self) -> Result<Vec<QualityScore>, VynFiError> {
        self.client.block_on(self.client.inner.quality().scores())
    }

    /// Get a daily quality score timeline.
    pub fn timeline(&self, days: Option<u32>) -> Result<Vec<DailyQuality>, VynFiError> {
        self.client
            .block_on(self.client.inner.quality().timeline(days))
    }
}

// ---------------------------------------------------------------------------
// Webhooks
// ---------------------------------------------------------------------------

/// Blocking handle for the Webhooks resource.
pub struct Webhooks<'a> {
    client: &'a Client,
}

impl Webhooks<'_> {
    /// Create a new webhook endpoint.
    pub fn create(&self, req: &CreateWebhookRequest) -> Result<WebhookCreated, VynFiError> {
        self.client
            .block_on(self.client.inner.webhooks().create(req))
    }

    /// List all webhooks.
    pub fn list(&self) -> Result<Vec<Webhook>, VynFiError> {
        self.client.block_on(self.client.inner.webhooks().list())
    }

    /// Get a single webhook by ID.
    pub fn get(&self, webhook_id: &str) -> Result<Webhook, VynFiError> {
        self.client
            .block_on(self.client.inner.webhooks().get(webhook_id))
    }

    /// Update a webhook's URL, events, or status.
    pub fn update(
        &self,
        webhook_id: &str,
        req: &UpdateWebhookRequest,
    ) -> Result<Webhook, VynFiError> {
        self.client
            .block_on(self.client.inner.webhooks().update(webhook_id, req))
    }

    /// Delete a webhook.
    pub fn delete(&self, webhook_id: &str) -> Result<(), VynFiError> {
        self.client
            .block_on(self.client.inner.webhooks().delete(webhook_id))
    }

    /// Send a test event to a webhook endpoint.
    pub fn test(&self, webhook_id: &str) -> Result<serde_json::Value, VynFiError> {
        self.client
            .block_on(self.client.inner.webhooks().test(webhook_id))
    }
}

// ---------------------------------------------------------------------------
// Billing
// ---------------------------------------------------------------------------

/// Blocking handle for the Billing resource.
pub struct Billing<'a> {
    client: &'a Client,
}

impl Billing<'_> {
    /// Get the current subscription details.
    pub fn subscription(&self) -> Result<Subscription, VynFiError> {
        self.client
            .block_on(self.client.inner.billing().subscription())
    }

    /// List all invoices.
    pub fn invoices(&self) -> Result<Vec<Invoice>, VynFiError> {
        self.client.block_on(self.client.inner.billing().invoices())
    }

    /// Get the payment method on file.
    pub fn payment_method(&self) -> Result<PaymentMethod, VynFiError> {
        self.client
            .block_on(self.client.inner.billing().payment_method())
    }
}
