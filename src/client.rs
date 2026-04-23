use std::time::Duration;

use reqwest::{RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::{ErrorBody, VynFiError};
use crate::resources::{
    Adversarial, Ai, ApiKeys, Billing, Catalog, Configs, Credits, Fingerprint, Jobs,
    Notifications, Optimizer, Quality, Scenarios, Sessions, TemplatePacks, Usage, Webhooks,
};

const DEFAULT_BASE_URL: &str = "https://api.vynfi.com";
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_MAX_RETRIES: u32 = 2;
const USER_AGENT: &str = concat!("vynfi-rust/", env!("CARGO_PKG_VERSION"));

/// Async VynFi API client.
///
/// Created via [`Client::builder`]. Holds a connection pool internally and is
/// cheap to clone — share a single instance across tasks.
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    base_url: String,
    max_retries: u32,
}

impl Client {
    /// Returns a [`ClientBuilder`] that accepts an API key.
    ///
    /// ```no_run
    /// # use vynfi::Client;
    /// let client = Client::builder("vf_live_abc123").build().unwrap();
    /// ```
    pub fn builder(api_key: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(api_key)
    }

    // -- Resource accessors ---------------------------------------------------

    /// Jobs resource — submit, list, get, cancel, stream, and download
    /// generation jobs.
    pub fn jobs(&self) -> Jobs<'_> {
        Jobs::new(self)
    }

    /// Catalog resource — list sectors, tables, and fingerprints.
    pub fn catalog(&self) -> Catalog<'_> {
        Catalog::new(self)
    }

    /// Usage resource — credit balance and daily usage breakdown.
    pub fn usage(&self) -> Usage<'_> {
        Usage::new(self)
    }

    /// API-key management resource.
    pub fn api_keys(&self) -> ApiKeys<'_> {
        ApiKeys::new(self)
    }

    /// Quality metrics resource.
    pub fn quality(&self) -> Quality<'_> {
        Quality::new(self)
    }

    /// Webhooks resource — CRUD and delivery history.
    pub fn webhooks(&self) -> Webhooks<'_> {
        Webhooks::new(self)
    }

    /// Billing resource — subscription, invoices, payment methods.
    pub fn billing(&self) -> Billing<'_> {
        Billing::new(self)
    }

    /// Configs resource — saved configs, validation, cost estimation.
    pub fn configs(&self) -> Configs<'_> {
        Configs::new(self)
    }

    /// Credits resource — prepaid credit packs and balances.
    pub fn credits(&self) -> Credits<'_> {
        Credits::new(self)
    }

    /// Sessions resource — multi-period generation sessions.
    pub fn sessions(&self) -> Sessions<'_> {
        Sessions::new(self)
    }

    /// Scenarios resource — what-if analysis scenarios.
    pub fn scenarios(&self) -> Scenarios<'_> {
        Scenarios::new(self)
    }

    /// Notifications resource — user notifications.
    pub fn notifications(&self) -> Notifications<'_> {
        Notifications::new(self)
    }

    /// Adversarial probing resource (Enterprise tier).
    pub fn adversarial(&self) -> Adversarial<'_> {
        Adversarial::new(self)
    }

    /// AI co-pilot resource (Scale+ tier).
    pub fn ai(&self) -> Ai<'_> {
        Ai::new(self)
    }

    /// Fingerprint synthesis resource (DS 3.0+, Team+).
    pub fn fingerprint(&self) -> Fingerprint<'_> {
        Fingerprint::new(self)
    }

    /// Audit optimizer resource (VynFi API 4.1+, Scale+).
    pub fn optimizer(&self) -> Optimizer<'_> {
        Optimizer::new(self)
    }

    /// Template packs resource (VynFi API 4.1+, Team+).
    pub fn template_packs(&self) -> TemplatePacks<'_> {
        TemplatePacks::new(self)
    }

    // -- Internal request helpers (used by resource structs) ------------------

    /// Make a JSON request (no body) with retry logic.
    pub(crate) async fn request<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> Result<T, VynFiError> {
        self.send_with_retry(method, path, |req| req).await
    }

    /// Make a JSON request with an optional body and retry logic.
    pub(crate) async fn request_with_body<T, B>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, VynFiError>
    where
        T: DeserializeOwned,
        B: serde::Serialize,
    {
        let body_value = body.map(|b| serde_json::to_value(b).expect("serializable body"));
        self.send_with_retry(method, path, move |req| match &body_value {
            Some(v) => req.json(v),
            None => req,
        })
        .await
    }

    /// Make a JSON request with query parameters and retry logic.
    pub(crate) async fn request_with_params<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<T, VynFiError> {
        let params = params.to_vec();
        self.send_with_retry(method, path, move |req| req.query(&params))
            .await
    }

    /// Raw response (for binary downloads), with optional query parameters.
    pub(crate) async fn request_raw(
        &self,
        method: reqwest::Method,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<Response, VynFiError> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.request(method, &url);
        if !params.is_empty() {
            req = req.query(params);
        }
        let resp = req.send().await?;
        if resp.status().is_client_error() || resp.status().is_server_error() {
            return Err(Self::error_from_response(resp).await);
        }
        Ok(resp)
    }

    /// Build an absolute URL for the given path (used by resources to open SSE
    /// streams via `reqwest-eventsource`).
    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Borrow the inner `reqwest::Client` (used by resources that need to
    /// construct custom requests, e.g. SSE streaming).
    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.http
    }

    // -- Private helpers ------------------------------------------------------

    /// Shared retry loop for all JSON-deserialized requests.
    async fn send_with_retry<T, F>(
        &self,
        method: reqwest::Method,
        path: &str,
        configure: F,
    ) -> Result<T, VynFiError>
    where
        T: DeserializeOwned,
        F: Fn(RequestBuilder) -> RequestBuilder,
    {
        let url = format!("{}{}", self.base_url, path);
        let mut last_err: Option<VynFiError> = None;

        for attempt in 0..=self.max_retries {
            let req = configure(self.http.request(method.clone(), &url));

            let resp = match req.send().await {
                Ok(r) => r,
                Err(e) => {
                    last_err = Some(VynFiError::Http(e));
                    if attempt < self.max_retries {
                        tokio::time::sleep(Self::backoff(attempt)).await;
                        continue;
                    }
                    return Err(last_err.unwrap());
                }
            };

            let status = resp.status();

            if Self::should_retry(status) && attempt < self.max_retries {
                let retry_after = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(Duration::from_secs);

                let wait = retry_after.unwrap_or_else(|| Self::backoff(attempt));
                let _ = resp.bytes().await;
                tokio::time::sleep(wait).await;
                continue;
            }

            if status == StatusCode::NO_CONTENT {
                return serde_json::from_value(serde_json::Value::Null).map_err(VynFiError::from);
            }

            if status.is_client_error() || status.is_server_error() {
                return Err(Self::error_from_response(resp).await);
            }

            let bytes = resp.bytes().await?;
            return serde_json::from_slice(&bytes).map_err(VynFiError::from);
        }

        Err(last_err.unwrap_or_else(|| VynFiError::Config("max retries exceeded".into())))
    }

    fn should_retry(status: StatusCode) -> bool {
        status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
    }

    /// Exponential back-off: 500ms, 1s, 2s, ...
    fn backoff(attempt: u32) -> Duration {
        Duration::from_millis(500 * 2u64.pow(attempt))
    }

    /// Convert an error HTTP response into the appropriate [`VynFiError`]
    /// variant based on its status code.
    async fn error_from_response(resp: Response) -> VynFiError {
        let status = resp.status();
        let body: ErrorBody = resp.json().await.unwrap_or_else(|_| ErrorBody {
            error_type: String::new(),
            title: String::new(),
            detail: format!("HTTP {}", status.as_u16()),
            status: status.as_u16(),
            instance: None,
        });

        let body = Box::new(body);
        match status {
            StatusCode::UNAUTHORIZED => VynFiError::Authentication(body),
            StatusCode::PAYMENT_REQUIRED => VynFiError::InsufficientCredits(body),
            StatusCode::FORBIDDEN => VynFiError::Forbidden(body),
            StatusCode::NOT_FOUND => VynFiError::NotFound(body),
            StatusCode::CONFLICT => VynFiError::Conflict(body),
            StatusCode::UNPROCESSABLE_ENTITY => VynFiError::Validation(body),
            StatusCode::TOO_MANY_REQUESTS => VynFiError::RateLimit(body),
            _ => VynFiError::Server(body),
        }
    }
}

/// Builder for configuring and constructing a [`Client`].
///
/// ```no_run
/// # use vynfi::Client;
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
    base_url: String,
    timeout: Duration,
    max_retries: u32,
}

impl ClientBuilder {
    fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }

    /// Override the base URL (trailing slashes are stripped).
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }

    /// Set the request timeout (default: 30 s).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the maximum number of automatic retries on 429 / 5xx (default: 2).
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Build the [`Client`]. Returns an error if the API key is empty or the
    /// underlying HTTP client cannot be constructed.
    pub fn build(self) -> Result<Client, VynFiError> {
        if self.api_key.is_empty() {
            return Err(VynFiError::Config("api_key is required".into()));
        }

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.api_key)
                .parse()
                .expect("valid authorization header value"),
        );
        headers.insert(
            reqwest::header::USER_AGENT,
            USER_AGENT.parse().expect("valid user-agent header value"),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json"
                .parse()
                .expect("valid content-type header value"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(self.timeout)
            .build()?;

        Ok(Client {
            http,
            base_url: self.base_url,
            max_retries: self.max_retries,
        })
    }
}

/// Extract a `Vec<T>` from an API response that may be a bare JSON array or an
/// object wrapping the array under `"data"` or another key.
pub(crate) fn extract_list<T: DeserializeOwned>(value: Value) -> Result<Vec<T>, VynFiError> {
    if value.is_array() {
        return Ok(serde_json::from_value(value)?);
    }
    if let Value::Object(mut map) = value {
        if let Some(arr) = map.remove("data").filter(|v| v.is_array()) {
            return Ok(serde_json::from_value(arr)?);
        }
        for (_, v) in map {
            if v.is_array() {
                return Ok(serde_json::from_value(v)?);
            }
        }
    }
    Ok(vec![])
}
