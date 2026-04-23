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
// Configs
// ---------------------------------------------------------------------------

/// A saved generation configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedConfig {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub description: String,
    pub config: serde_json::Value,
    pub source_template_id: Option<String>,
    #[serde(default)]
    pub version: i32,
    pub visibility: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub schema_version: Option<i32>,
}

/// Request body for creating a saved config.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConfigRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub config: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Request body for updating a saved config.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Response from deleting a config.
#[derive(Debug, Clone, Deserialize)]
pub struct DeletedResponse {
    pub deleted: bool,
}

/// Request body for validating a config.
#[derive(Debug, Clone, Serialize)]
pub struct ValidateConfigRequest {
    pub config: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
}

/// A validation issue (error or warning).
#[derive(Debug, Clone, Deserialize)]
pub struct ValidationIssue {
    pub field: String,
    pub code: String,
    pub message: String,
    pub fix: Option<ValidationFix>,
}

/// A suggested fix for a validation issue.
#[derive(Debug, Clone, Deserialize)]
pub struct ValidationFix {
    pub field: String,
    pub action: String,
    pub value: serde_json::Value,
}

/// Response from config validation.
#[derive(Debug, Clone, Deserialize)]
pub struct ValidateConfigResponse {
    pub valid: bool,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

/// Request body for estimating config cost.
#[derive(Debug, Clone, Serialize)]
pub struct EstimateCostRequest {
    pub config: serde_json::Value,
}

/// A credit multiplier entry in a cost estimate.
#[derive(Debug, Clone, Deserialize)]
pub struct MultiplierEntry {
    pub source: String,
    pub factor: f64,
    pub label: String,
}

/// Balance information in a cost estimate.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceInfo {
    pub current: i64,
    pub after_job: i64,
    pub status: String,
}

/// Response from estimating config cost.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateCostResponse {
    pub base_credits: i64,
    pub multipliers: Vec<MultiplierEntry>,
    pub total_credits: i64,
    pub capped_at: Option<f64>,
    pub balance: BalanceInfo,
}

/// Request body for composing a config from layers.
#[derive(Debug, Clone, Serialize)]
pub struct ComposeConfigRequest {
    pub layers: Vec<serde_json::Value>,
}

/// Response from composing a config.
#[derive(Debug, Clone, Deserialize)]
pub struct ComposeConfigResponse {
    pub config: serde_json::Value,
    pub yaml: String,
    pub layers: Vec<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Credits
// ---------------------------------------------------------------------------

/// Request body for purchasing a prepaid credit pack.
#[derive(Debug, Clone, Serialize)]
pub struct PurchaseCreditsRequest {
    pub pack: String,
}

/// Response from purchasing credits (Stripe checkout URL).
#[derive(Debug, Clone, Deserialize)]
pub struct PurchaseCreditsResponse {
    pub checkout_url: String,
}

/// A prepaid credit batch.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepaidBatch {
    pub id: String,
    pub owner_id: String,
    pub pack: String,
    pub credits_purchased: i64,
    pub credits_remaining: i64,
    pub credits_forfeited: i64,
    pub status: String,
    pub purchased_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Prepaid credit balance with active batches.
#[derive(Debug, Clone, Deserialize)]
pub struct PrepaidBalanceResponse {
    pub total_prepaid_credits: i64,
    pub batches: Vec<PrepaidBatch>,
}

/// Prepaid credit history (includes expired batches).
#[derive(Debug, Clone, Deserialize)]
pub struct PrepaidHistoryResponse {
    pub batches: Vec<PrepaidBatch>,
}

// ---------------------------------------------------------------------------
// Sessions
// ---------------------------------------------------------------------------

/// Request body for creating a multi-period generation session.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionRequest {
    pub name: String,
    pub fiscal_year_start: String,
    pub period_length_months: i32,
    pub periods: i32,
    pub generation_config: serde_json::Value,
}

/// Request body for extending a session with additional periods.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtendSessionRequest {
    pub additional_periods: i32,
}

/// A multi-period generation session.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationSession {
    pub id: String,
    pub name: String,
    pub status: String,
    pub fiscal_year_start: String,
    pub period_length_months: i32,
    pub periods_total: i32,
    pub periods_generated: i32,
    pub periods: serde_json::Value,
    pub balance_snapshot: Option<serde_json::Value>,
    pub generation_config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response from generating the next period of a session.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateSessionResponse {
    #[serde(flatten)]
    pub session: GenerationSession,
    pub job_id: String,
    pub period_index: i32,
    pub credits_reserved: i64,
    pub period_start: String,
    pub period_end: String,
}

// ---------------------------------------------------------------------------
// Templates
// ---------------------------------------------------------------------------

/// A system template for generation configs.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub sector: String,
    pub country: String,
    pub framework: String,
    pub config: serde_json::Value,
    pub min_tier: String,
    pub sort_order: i32,
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

/// Request body for creating a what-if scenario.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateScenarioRequest {
    pub name: String,
    pub template_id: String,
    pub interventions: serde_json::Value,
    pub generation_config: serde_json::Value,
}

/// A what-if scenario comparing baseline and counterfactual generation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub template_id: String,
    pub status: String,
    pub interventions: serde_json::Value,
    pub generation_config: serde_json::Value,
    pub baseline_job_id: Option<String>,
    pub counterfactual_job_id: Option<String>,
    pub diff: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A scenario template with graph structure.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub node_count: i32,
    pub nodes: Vec<ScenarioTemplateNode>,
    pub edges: Vec<ScenarioTemplateEdge>,
    pub intervention_types: Vec<String>,
}

/// A node in a scenario template graph.
#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioTemplateNode {
    pub id: String,
    pub label: String,
    pub x: i32,
    pub y: i32,
}

/// An edge in a scenario template graph.
#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioTemplateEdge {
    pub source: String,
    pub target: String,
}

// ---------------------------------------------------------------------------
// Notifications
// ---------------------------------------------------------------------------

/// A user notification.
#[derive(Debug, Clone, Deserialize)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub link: Option<String>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

/// Request body for marking notifications as read.
#[derive(Debug, Clone, Serialize)]
pub struct MarkReadRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
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

// ---------------------------------------------------------------------------
// Analytics (DataSynth 2.3+, v1.8.0)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BenfordAnalysis {
    #[serde(default)]
    pub sample_size: i64,
    #[serde(default)]
    pub observed_frequencies: Vec<f64>,
    #[serde(default)]
    pub observed_counts: Vec<i64>,
    #[serde(default)]
    pub expected_frequencies: Vec<f64>,
    #[serde(default)]
    pub chi_squared: f64,
    #[serde(default)]
    pub degrees_of_freedom: i64,
    #[serde(default)]
    pub p_value: f64,
    #[serde(default)]
    pub mad: f64,
    #[serde(default)]
    pub conformity: String,
    #[serde(default)]
    pub passes: bool,
    #[serde(default)]
    pub anti_benford_score: f64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AmountDistributionAnalysis {
    #[serde(default)]
    pub sample_size: i64,
    #[serde(default)]
    pub mean: String,
    #[serde(default)]
    pub median: String,
    #[serde(default)]
    pub std_dev: String,
    #[serde(default)]
    pub min: String,
    #[serde(default)]
    pub max: String,
    #[serde(default)]
    pub percentile_1: String,
    #[serde(default)]
    pub percentile_99: String,
    #[serde(default)]
    pub skewness: f64,
    #[serde(default)]
    pub kurtosis: f64,
    #[serde(default)]
    pub round_number_ratio: f64,
    #[serde(default)]
    pub nice_number_ratio: f64,
    #[serde(default)]
    pub passes: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct VariantAnalysis {
    #[serde(default)]
    pub variant_count: i64,
    #[serde(default)]
    pub total_cases: i64,
    #[serde(default)]
    pub variant_entropy: f64,
    #[serde(default)]
    pub happy_path_concentration: f64,
    #[serde(default)]
    pub passes: bool,
    #[serde(default)]
    pub issues: Vec<String>,
    #[serde(default)]
    pub rework_rate: f64,
    #[serde(default)]
    pub skipped_step_rate: f64,
    #[serde(default)]
    pub out_of_order_rate: f64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TypologyDetection {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub transaction_count: i64,
    #[serde(default)]
    pub case_count: i64,
    #[serde(default)]
    pub flag_rate: f64,
    #[serde(default)]
    pub pattern_detected: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct KycCompletenessAnalysis {
    #[serde(default)]
    pub core_field_rate: f64,
    #[serde(default)]
    pub name_rate: f64,
    #[serde(default)]
    pub dob_rate: f64,
    #[serde(default)]
    pub address_rate: f64,
    #[serde(default)]
    pub id_document_rate: f64,
    #[serde(default)]
    pub risk_rating_rate: f64,
    #[serde(default)]
    pub total_profiles: i64,
    #[serde(default)]
    pub passes: bool,
    #[serde(default)]
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AmlDetectabilityAnalysis {
    #[serde(default)]
    pub typology_coverage: f64,
    #[serde(default)]
    pub scenario_coherence: f64,
    #[serde(default)]
    pub per_typology: Vec<TypologyDetection>,
    #[serde(default)]
    pub total_transactions: i64,
    #[serde(default)]
    pub passes: bool,
    #[serde(default)]
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BankingEvaluation {
    pub kyc: Option<KycCompletenessAnalysis>,
    pub aml: Option<AmlDetectabilityAnalysis>,
    #[serde(default)]
    pub passes: bool,
    #[serde(default)]
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct JobAnalytics {
    pub benford_analysis: Option<BenfordAnalysis>,
    pub amount_distribution: Option<AmountDistributionAnalysis>,
    pub process_variant_summary: Option<VariantAnalysis>,
    pub banking_evaluation: Option<BankingEvaluation>,
}

// ---------------------------------------------------------------------------
// Audit (DS 3.1.0+)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AuditOpinion {
    #[serde(default)]
    pub opinion_id: String,
    #[serde(default)]
    pub company_code: String,
    pub fiscal_year: Option<i32>,
    #[serde(default)]
    pub opinion_type: String,
    #[serde(default)]
    pub going_concern: String,
    #[serde(default)]
    pub basis_for_opinion: String,
    #[serde(default)]
    pub signed_by: String,
    #[serde(default)]
    pub signed_date: String,
    #[serde(default)]
    pub matters: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct KeyAuditMatter {
    #[serde(default)]
    pub matter_id: String,
    #[serde(default)]
    pub company_code: String,
    pub fiscal_year: Option<i32>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub audit_response: String,
    #[serde(default)]
    pub related_accounts: Vec<String>,
    #[serde(default)]
    pub risk_level: String,
}

// ---------------------------------------------------------------------------
// Fraud split (DS 3.1.1+)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FraudTypeSplit {
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub scheme_propagated: i64,
    #[serde(default)]
    pub direct_injection: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FraudSplit {
    #[serde(default)]
    pub total_entries: i64,
    #[serde(default)]
    pub fraud_entries: i64,
    #[serde(default)]
    pub scheme_propagated: i64,
    #[serde(default)]
    pub direct_injection: i64,
    #[serde(default)]
    pub propagation_rate: f64,
    #[serde(default)]
    pub by_fraud_type: std::collections::HashMap<String, FraudTypeSplit>,
}

// ---------------------------------------------------------------------------
// Audit artifacts (API 4.1+)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AuditArtifacts {
    pub audit_opinions: Option<serde_json::Value>,
    pub key_audit_matters: Option<serde_json::Value>,
    pub anomaly_labels: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// File listing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileSchema {
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type", default)]
    pub type_: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobFile {
    pub path: String,
    #[serde(default)]
    pub size_bytes: i64,
    #[serde(default)]
    pub content_type: String,
    #[serde(default, rename = "schema")]
    pub schema_: Vec<FileSchema>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobFileList {
    #[serde(default)]
    pub job_id: String,
    #[serde(default)]
    pub total_files: i64,
    #[serde(default)]
    pub total_size_bytes: i64,
    #[serde(default)]
    pub files: Vec<JobFile>,
}

// ---------------------------------------------------------------------------
// Optimizer (VynFi API 4.1+, DS 4.1.2+)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OptimizerResponse {
    pub report: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RiskScopeRequest {
    pub engagement: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioRequest {
    pub candidates: serde_json::Value,
    pub budget_hours: u32,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesRequest {
    pub schedule: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConformanceRequest {
    pub trace: serde_json::Value,
    pub blueprint: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MonteCarloRequest {
    pub engagement: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CalibrationRequest {
    pub findings: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Template packs (VynFi API 4.1+, DS 3.2+, Team+)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TemplatePackCategorySummary {
    pub category: String,
    #[serde(default)]
    pub size_bytes: i64,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TemplatePack {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_merge_strategy")]
    pub merge_strategy: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub categories: Vec<TemplatePackCategorySummary>,
}

fn default_merge_strategy() -> String {
    "extend".into()
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TemplatePackList {
    #[serde(default)]
    pub packs: Vec<TemplatePack>,
    #[serde(default)]
    pub limit: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TemplatePackValidationIssue {
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TemplatePackValidation {
    #[serde(default = "default_true")]
    pub valid: bool,
    #[serde(default)]
    pub categories_checked: Vec<String>,
    #[serde(default)]
    pub issues: Vec<TemplatePackValidationIssue>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TemplatePackCategoryContent {
    pub category: String,
    #[serde(default)]
    pub content_yaml: String,
    #[serde(default)]
    pub size_bytes: i64,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TemplatePackEnrichResponse {
    pub category: String,
    pub target_pack_category: String,
    #[serde(default)]
    pub count_requested: u32,
    #[serde(default)]
    pub size_bytes_after: i32,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreatePackRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePackRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnrichCategoryRequest {
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_pack_category: Option<String>,
}

// ---------------------------------------------------------------------------
// NL config (VynFi API 4.1+)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NlConfigResponse {
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub yaml: String,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompanyConfigResponse {
    pub company: Option<serde_json::Value>,
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub yaml: String,
    #[serde(default)]
    pub notes: String,
}

// ---------------------------------------------------------------------------
// SAP / SAF-T export (VynFi API 4.4+, DS 4.3+)
// ---------------------------------------------------------------------------

/// Default 8-table SAP set emitted when `exportFormat == "sap"` without an
/// explicit `tables` list.
pub const SAP_DEFAULT_TABLES: &[&str] = &[
    "bkpf", "bseg", "acdoca", "lfa1", "kna1", "mara", "csks", "cepc",
];

/// Full SAP superset (DS 4.3+) — master data, transactional, open/cleared
/// items for GL/AR/AP.
pub const SAP_ALL_TABLES: &[&str] = &[
    "bkpf", "bseg", "acdoca",
    "lfa1", "lfb1", "kna1", "knb1", "mara", "mard", "anla", "csks", "cepc",
    "ska1", "skb1",
    "ekko", "ekpo", "vbak", "vbap", "likp", "lips", "mkpf", "mseg",
    "bsis", "bsas", "bsid", "bsad", "bsik", "bsak",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SapExportConfig {
    #[serde(default = "default_sap_dialect")]
    pub dialect: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tables: Vec<String>,
    #[serde(default = "default_sap_client")]
    pub client: String,
    #[serde(default = "default_sap_ledger")]
    pub ledger: String,
    #[serde(default = "default_sap_source_system")]
    pub source_system: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_currency: Option<String>,
    #[serde(default = "default_true")]
    pub include_extension_fields: bool,
}

fn default_sap_dialect() -> String { "hana".into() }
fn default_sap_client() -> String { "200".into() }
fn default_sap_ledger() -> String { "0L".into() }
fn default_sap_source_system() -> String { "DATASYNTH".into() }

impl Default for SapExportConfig {
    fn default() -> Self {
        Self {
            dialect: default_sap_dialect(),
            tables: Vec::new(),
            client: default_sap_client(),
            ledger: default_sap_ledger(),
            source_system: default_sap_source_system(),
            local_currency: None,
            include_extension_fields: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaftExportConfig {
    pub jurisdiction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_tax_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,
}

impl SaftExportConfig {
    pub fn new(jurisdiction: impl Into<String>) -> Self {
        Self {
            jurisdiction: jurisdiction.into(),
            company_tax_id: None,
            company_name: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ChartOfAccountsMeta {
    pub coa_id: Option<String>,
    pub accounting_framework: Option<String>,
    pub country: Option<String>,
    pub industry: Option<String>,
    pub complexity: Option<String>,
    pub account_count: Option<i64>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Adversarial (DS 3.0+, Enterprise)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AdversarialProbeResponse {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProbeSample {
    #[serde(default)]
    pub id: String,
    pub prediction: Option<serde_json::Value>,
    pub ground_truth: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AdversarialProbeResults {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub samples: Vec<ProbeSample>,
    pub metrics: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// AI (DS 3.0+, Scale+)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AiTuneResponse {
    pub original_config: Option<serde_json::Value>,
    pub suggested_config: Option<serde_json::Value>,
    #[serde(default)]
    pub explanation: String,
    pub quality_summary: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AiChatResponse {
    #[serde(default)]
    pub reply: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AiChatRequest {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AiTuneRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_scores: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<u32>,
}

// ---------------------------------------------------------------------------
// Fingerprint (DS 3.0+, Team+)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FingerprintSynthesisResponse {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub status: String,
}
