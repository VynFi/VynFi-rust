use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Jobs
// ---------------------------------------------------------------------------

/// Links returned with a submitted job.
#[derive(Debug, Clone, Deserialize)]
pub struct JobLinks {
    #[serde(rename = "self", default)]
    pub self_link: String,
    #[serde(default)]
    pub stream: String,
    #[serde(default)]
    pub cancel: String,
}

/// A generation job.
#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    pub id: String,
    pub owner_id: Option<String>,
    pub status: String,
    pub config: Option<serde_json::Value>,
    pub progress: Option<serde_json::Value>,
    pub credits_reserved: i64,
    pub credits_used: Option<i64>,
    pub artifacts: Option<serde_json::Value>,
    pub error_detail: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Response from submitting an async generation job.
#[derive(Debug, Clone, Deserialize)]
pub struct SubmitJobResponse {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub credits_reserved: i64,
    #[serde(default)]
    pub estimated_duration_seconds: i64,
    pub links: Option<JobLinks>,
}

/// Response from a quick (synchronous) generation job.
#[derive(Debug, Clone, Deserialize)]
pub struct QuickJobResponse {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub credits_used: i64,
    #[serde(default)]
    pub rows_generated: i64,
    pub download_url: Option<String>,
}

/// Response from cancelling a job.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelJobResponse {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub credits_reserved: i64,
    #[serde(default)]
    pub credits_used: i64,
    #[serde(default)]
    pub credits_refunded: i64,
    #[serde(default)]
    pub rows_generated: i64,
    #[serde(default)]
    pub rows_total: i64,
}

/// Paginated list of jobs.
#[derive(Debug, Clone, Deserialize)]
pub struct JobList {
    pub data: Vec<Job>,
}

/// A table specification within a legacy generation request.
#[derive(Debug, Clone, Serialize)]
pub struct TableSpec {
    pub name: String,
    pub rows: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_rate: Option<f64>,
}

/// Legacy generation request (tables-based).
#[derive(Debug, Clone, Serialize)]
pub struct GenerateRequest {
    pub tables: Vec<TableSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector_slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
}

impl GenerateRequest {
    /// Create a new legacy generate request with sensible defaults.
    pub fn new(tables: Vec<TableSpec>, sector_slug: impl Into<String>) -> Self {
        Self {
            tables,
            format: None,
            sector_slug: Some(sector_slug.into()),
            options: None,
        }
    }
}

/// Config-based generation request (portal-style).
#[derive(Debug, Clone, Serialize)]
pub struct GenerateConfigRequest {
    pub config: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_id: Option<String>,
}

/// A parsed Server-Sent Event from the job progress stream.
#[derive(Debug, Clone)]
pub struct SseEvent {
    /// The event type (e.g. `"progress"`, `"complete"`, `"error"`).
    pub event: String,
    /// The JSON payload.
    pub data: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Catalog / Sectors
// ---------------------------------------------------------------------------

/// A column definition within a table.
#[derive(Debug, Clone, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub description: String,
    pub nullable: bool,
    #[serde(default)]
    pub example_values: Option<Vec<String>>,
}

/// A table definition within a sector.
#[derive(Debug, Clone, Deserialize)]
pub struct TableDef {
    pub id: Option<String>,
    pub slug: Option<String>,
    pub name: String,
    pub description: String,
    #[serde(default = "default_base_rate")]
    pub base_rate: f64,
    pub columns: Vec<Column>,
}

fn default_base_rate() -> f64 {
    1.0
}

/// A full sector with its tables (from GET /v1/sectors/{slug}).
#[derive(Debug, Clone, Deserialize)]
pub struct Sector {
    pub id: Option<String>,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
    pub quality_score: i32,
    pub popularity: i32,
    pub tables: Vec<TableDef>,
}

fn default_multiplier() -> f64 {
    1.0
}

/// Abbreviated sector information (from GET /v1/sectors).
#[derive(Debug, Clone, Deserialize)]
pub struct SectorSummary {
    pub id: Option<String>,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
    pub quality_score: i32,
    pub popularity: i32,
    pub table_count: i64,
}

/// A catalog item (from GET /v1/catalog).
#[derive(Debug, Clone, Deserialize)]
pub struct CatalogItem {
    pub id: Option<String>,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
    pub quality_score: i32,
    pub popularity: i32,
    pub table_count: i64,
}

/// A fingerprint detail (from GET /v1/catalog/{sector}/{profile}).
#[derive(Debug, Clone, Deserialize)]
pub struct Fingerprint {
    pub sector: serde_json::Value,
    pub table: TableDef,
}

// ---------------------------------------------------------------------------
// Usage
// ---------------------------------------------------------------------------

/// Credit usage summary.
#[derive(Debug, Clone, Deserialize)]
pub struct UsageSummary {
    pub balance: i64,
    pub total_used: i64,
    pub total_reserved: i64,
    pub total_refunded: i64,
    pub period_days: i32,
    pub burn_rate: i64,
}

/// Credits consumed on a single day.
#[derive(Debug, Clone, Deserialize)]
pub struct DailyUsage {
    pub date: NaiveDate,
    pub credits: i64,
}

/// Per-table usage breakdown.
#[derive(Debug, Clone, Deserialize)]
pub struct TableUsage {
    pub table_name: String,
    pub credits: i64,
    pub job_count: i64,
}

/// Daily usage response with per-table totals.
#[derive(Debug, Clone, Deserialize)]
pub struct DailyUsageResponse {
    pub daily: Vec<DailyUsage>,
    pub by_table: Vec<TableUsage>,
}

// ---------------------------------------------------------------------------
// API Keys
// ---------------------------------------------------------------------------

/// An existing API key (secret not included).
#[derive(Debug, Clone, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub environment: String,
    #[serde(default = "default_active")]
    pub status: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

fn default_active() -> String {
    "active".to_string()
}

/// A newly created API key (includes the full secret).
#[derive(Debug, Clone, Deserialize)]
pub struct ApiKeyCreated {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub key: String,
    pub environment: String,
    pub created_at: Option<DateTime<Utc>>,
}

/// Request body for creating an API key.
#[derive(Debug, Clone, Serialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

/// Request body for updating an API key.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateApiKeyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<serde_json::Value>,
}

/// Response from revoking an API key.
#[derive(Debug, Clone, Deserialize)]
pub struct RevokeKeyResponse {
    pub id: String,
    pub status: String,
    pub revoked_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Quality
// ---------------------------------------------------------------------------

/// Quality score for a generated table.
#[derive(Debug, Clone, Deserialize)]
pub struct QualityScore {
    pub id: String,
    pub job_id: String,
    pub table_type: String,
    pub rows: i32,
    pub overall_score: f32,
    pub benford_score: f32,
    pub correlation_score: f32,
    pub distribution_score: f32,
    pub created_at: Option<DateTime<Utc>>,
}

/// Aggregate quality score for a single day.
#[derive(Debug, Clone, Deserialize)]
pub struct DailyQuality {
    pub date: NaiveDate,
    pub score: f64,
}

// ---------------------------------------------------------------------------
// Webhooks
// ---------------------------------------------------------------------------

/// An existing webhook (list view).
#[derive(Debug, Clone, Deserialize)]
pub struct Webhook {
    pub id: String,
    pub url: String,
    pub events: Vec<String>,
    #[serde(default = "default_active")]
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
}

/// A newly created webhook (includes the signing secret).
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookCreated {
    pub id: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: String,
    #[serde(default = "default_active")]
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
}

/// Webhook detail with delivery history.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookDetail {
    pub id: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub deliveries: Vec<WebhookDelivery>,
}

/// A single webhook delivery attempt.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookDelivery {
    pub id: String,
    pub webhook_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status_code: Option<i32>,
    pub response_body: Option<String>,
    pub attempt: i32,
    pub succeeded: bool,
    pub created_at: Option<DateTime<Utc>>,
}

/// Request body for creating a webhook.
#[derive(Debug, Clone, Serialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub events: Vec<String>,
}

/// Request body for updating a webhook.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateWebhookRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// Billing
// ---------------------------------------------------------------------------

/// Subscription details.
#[derive(Debug, Clone, Deserialize)]
pub struct Subscription {
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub status: String,
    pub stripe_price_id: Option<String>,
    pub current_period_end: Option<DateTime<Utc>>,
}

/// Request body for creating a Stripe checkout session.
#[derive(Debug, Clone, Serialize)]
pub struct CheckoutRequest {
    pub price_id: String,
}

/// Response containing a Stripe checkout URL.
#[derive(Debug, Clone, Deserialize)]
pub struct CheckoutResponse {
    pub checkout_url: String,
}

/// Response containing a Stripe billing portal URL.
#[derive(Debug, Clone, Deserialize)]
pub struct PortalResponse {
    pub portal_url: String,
}

/// An invoice from Stripe.
#[derive(Debug, Clone, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub number: Option<String>,
    pub amount_due: i64,
    pub amount_paid: i64,
    pub status: String,
    pub created: i64,
    pub due_date: Option<i64>,
    pub hosted_invoice_url: Option<String>,
    pub pdf: Option<String>,
}
