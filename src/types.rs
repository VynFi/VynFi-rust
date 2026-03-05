use std::collections::HashMap;

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
    #[serde(default)]
    pub download: String,
}

/// Progress information for a running job.
#[derive(Debug, Clone, Deserialize)]
pub struct JobProgress {
    #[serde(default)]
    pub percent: u32,
    #[serde(default)]
    pub rows_generated: u64,
    #[serde(default)]
    pub rows_total: u64,
}

/// A generation job.
#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    pub id: String,
    pub status: String,
    pub tables: Option<serde_json::Value>,
    #[serde(default = "default_format")]
    pub format: String,
    pub credits_reserved: Option<i64>,
    pub credits_used: Option<i64>,
    pub sector_slug: String,
    pub progress: Option<JobProgress>,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Response from submitting a new job.
#[derive(Debug, Clone, Deserialize)]
pub struct SubmitJobResponse {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub credits_reserved: i64,
    #[serde(default)]
    pub estimated_duration_seconds: u64,
    pub links: Option<JobLinks>,
}

/// Paginated list of jobs.
#[derive(Debug, Clone, Deserialize)]
pub struct JobList {
    pub jobs: Vec<Job>,
    #[serde(default)]
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

/// A table specification within a generation request.
#[derive(Debug, Clone, Serialize)]
pub struct TableSpec {
    pub name: String,
    pub rows: u64,
}

/// Request body for generating synthetic data.
#[derive(Debug, Clone, Serialize)]
pub struct GenerateRequest {
    pub tables: Vec<TableSpec>,
    pub format: String,
    pub sector_slug: String,
}

impl GenerateRequest {
    /// Create a new generate request with sensible defaults.
    pub fn new(tables: Vec<TableSpec>) -> Self {
        Self {
            tables,
            format: "json".to_string(),
            sector_slug: "retail".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Catalog
// ---------------------------------------------------------------------------

/// A column definition within a table.
#[derive(Debug, Clone, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub description: String,
    pub nullable: bool,
}

/// A table definition within a sector.
#[derive(Debug, Clone, Deserialize)]
pub struct TableDef {
    pub name: String,
    pub description: String,
    #[serde(default = "default_base_rate")]
    pub base_rate: f64,
    pub columns: Vec<Column>,
}

fn default_base_rate() -> f64 {
    1.0
}

/// A full sector with its tables.
#[derive(Debug, Clone, Deserialize)]
pub struct Sector {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
    pub quality_score: f64,
    pub popularity: u32,
    pub tables: Vec<TableDef>,
}

fn default_multiplier() -> f64 {
    1.0
}

fn default_format() -> String {
    "json".to_string()
}

/// Abbreviated sector information (no tables).
#[derive(Debug, Clone, Deserialize)]
pub struct SectorSummary {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub table_count: u32,
}

/// A catalog item.
#[derive(Debug, Clone, Deserialize)]
pub struct CatalogItem {
    pub sector: String,
    pub profile: String,
    pub name: String,
    pub description: String,
    pub source: String,
}

/// A fingerprint definition with column details.
#[derive(Debug, Clone, Deserialize)]
pub struct Fingerprint {
    pub sector: String,
    pub profile: String,
    pub name: String,
    pub description: String,
    pub source: String,
    pub columns: Vec<Column>,
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
    pub burn_rate: f64,
    #[serde(default = "default_period_days")]
    pub period_days: u32,
}

fn default_period_days() -> u32 {
    30
}

/// Credits consumed on a single day.
#[derive(Debug, Clone, Deserialize)]
pub struct DailyUsage {
    pub date: NaiveDate,
    pub credits: i64,
}

/// Daily usage breakdown with per-table totals.
#[derive(Debug, Clone, Deserialize)]
pub struct DailyUsageResponse {
    pub daily: Vec<DailyUsage>,
    pub by_table: HashMap<String, i64>,
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
    pub scopes: Vec<String>,
    #[serde(default = "default_active")]
    pub status: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
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
    pub key: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Request body for creating an API key.
#[derive(Debug, Clone, Serialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub scopes: Option<Vec<String>>,
    pub expires_in_days: Option<u32>,
}

/// Request body for updating an API key.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateApiKeyRequest {
    pub name: Option<String>,
    pub scopes: Option<Vec<String>>,
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
    pub rows: u64,
    pub overall_score: f64,
    pub benford_score: f64,
    pub correlation_score: f64,
    pub distribution_score: f64,
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

/// An existing webhook.
#[derive(Debug, Clone, Deserialize)]
pub struct Webhook {
    pub id: String,
    pub url: String,
    pub events: Vec<String>,
    #[serde(default = "default_active")]
    pub status: String,
    pub secret: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// A newly created webhook (includes the signing secret).
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookCreated {
    pub id: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: String,
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
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// Billing
// ---------------------------------------------------------------------------

/// Subscription details.
#[derive(Debug, Clone, Deserialize)]
pub struct Subscription {
    #[serde(default = "default_free")]
    pub tier: String,
    #[serde(default = "default_active")]
    pub status: String,
    pub current_period_end: Option<DateTime<Utc>>,
    #[serde(default)]
    pub cancel_at_period_end: bool,
}

fn default_free() -> String {
    "free".to_string()
}

/// An invoice.
#[derive(Debug, Clone, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub amount: i64,
    #[serde(default = "default_usd")]
    pub currency: String,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub pdf_url: Option<String>,
}

fn default_usd() -> String {
    "usd".to_string()
}

/// A payment method on file.
#[derive(Debug, Clone, Deserialize)]
pub struct PaymentMethod {
    pub r#type: String,
    pub brand: String,
    pub last4: String,
    pub exp_month: u32,
    pub exp_year: u32,
}
