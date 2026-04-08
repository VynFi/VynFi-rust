use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Jobs
// ---------------------------------------------------------------------------

/// A generation job.
#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    pub id: String,
    pub user_id: Option<String>,
    pub status: String,
    pub tables: Option<serde_json::Value>,
    #[serde(default = "default_format")]
    pub format: String,
    pub sector_slug: String,
    pub rows_requested: Option<i64>,
    pub rows_generated: Option<i64>,
    pub credits_reserved: Option<i64>,
    pub credits_used: Option<i64>,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Response from submitting an async generation job.
#[derive(Debug, Clone, Deserialize)]
pub struct SubmitJobResponse {
    pub object: Option<String>,
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub credits_reserved: i64,
    pub message: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    pub sector_slug: String,
}

impl GenerateRequest {
    /// Create a new generate request with sensible defaults.
    pub fn new(tables: Vec<TableSpec>, sector_slug: impl Into<String>) -> Self {
        Self {
            tables,
            format: None,
            sector_slug: sector_slug.into(),
        }
    }
}

/// Response from the download endpoint (presigned URL).
#[derive(Debug, Clone, Deserialize)]
pub struct DownloadResponse {
    pub object: Option<String>,
    pub url: String,
    pub expires_in: u64,
}

fn default_format() -> String {
    "json".to_string()
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

/// Abbreviated sector information (no tables).
#[derive(Debug, Clone, Deserialize)]
pub struct SectorSummary {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
    pub quality_score: Option<f64>,
    pub popularity: Option<u32>,
    pub table_count: u32,
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
    #[serde(default)]
    pub tier: String,
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
    #[serde(default)]
    pub environment: String,
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
    #[serde(default)]
    pub environment: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Request body for creating an API key.
#[derive(Debug, Clone, Serialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in_days: Option<u32>,
}

/// Request body for updating an API key.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateApiKeyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Credits
// ---------------------------------------------------------------------------

/// Request body for purchasing a credit pack.
#[derive(Debug, Clone, Serialize)]
pub struct PurchaseCreditsRequest {
    pub pack: String,
}

/// Response from purchasing credits (Stripe checkout URL).
#[derive(Debug, Clone, Deserialize)]
pub struct PurchaseCreditsResponse {
    pub checkout_url: String,
}

/// A single prepaid credit batch.
#[derive(Debug, Clone, Deserialize)]
pub struct CreditBatch {
    pub batch_id: String,
    pub pack: String,
    pub credits_remaining: i64,
    pub credits_purchased: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: String,
}

/// Prepaid credit balance.
#[derive(Debug, Clone, Deserialize)]
pub struct CreditBalance {
    pub total_prepaid_credits: i64,
    pub batches: Vec<CreditBatch>,
}

/// A credit batch with full history details.
#[derive(Debug, Clone, Deserialize)]
pub struct CreditHistoryBatch {
    pub batch_id: String,
    pub user_id: Option<String>,
    pub pack: String,
    pub credits_purchased: i64,
    pub credits_remaining: i64,
    #[serde(default)]
    pub credits_forfeited: i64,
    pub status: String,
    pub purchased_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub stripe_payment_id: Option<String>,
    pub refund_id: Option<String>,
    pub dispute_id: Option<String>,
}

/// Credit purchase history.
#[derive(Debug, Clone, Deserialize)]
pub struct CreditHistory {
    pub batches: Vec<CreditHistoryBatch>,
}
