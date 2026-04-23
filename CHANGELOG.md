# Changelog

All notable changes to this project will be documented in this file.

## [1.8.0] - 2026-04-23

Bulk catch-up to the Python SDK master (v1.8.0). Every resource, helper,
and type the Python SDK exposes is now on the Rust side.

### Added — new resources

- **`Adversarial`** (`client.adversarial()`, Enterprise) — `probe()` / `results()`.
- **`Ai`** (`client.ai()`, Scale+) — `chat()` co-pilot.
- **`Fingerprint`** (`client.fingerprint()`, Team+) — `synthesize()`.
- **`Optimizer`** (`client.optimizer()`, Scale+) — six `POST /v1/optimizer/*` wrappers.
- **`TemplatePacks`** (`client.template_packs()`, Team+) — full CRUD + validate + enrich + categories.

### Added — Jobs gap-filling

- `analytics()`, `fraud_split()`, `audit_artifacts()`, `list_files()` (404-retry),
  `download_archive()` → `JobArchive`, `download_to(path)`, `stream_ndjson()`,
  `tune()`, `wait()`, `wait_for_many()`

### Added — Configs gap-filling

- `from_description(text)` (Scale+), `from_company(req)` (Scale+),
  `estimate_size(config)`, `submit_raw(yaml)` (Scale+)

### Added — `JobArchive`

Zip + managed_blob backends, transparent lazy-fetch. Matches the Python
`JobArchive` API: files, find, categories, read, text, json, size, url,
extract_to, audit_opinions, key_audit_matters, sap_tables, sap_table,
saft_file (root + legacy nested), coa_meta.

### Added — types

~650 lines in `types.rs`: analytics (BenfordAnalysis, AmountDistribution,
VariantAnalysis, KycCompleteness, AmlDetectability, BankingEvaluation,
JobAnalytics), audit (AuditOpinion, KeyAuditMatter, AuditArtifacts),
fraud-split, file listing, optimizer request+response set, template
packs set, NL config responses, AI types, fingerprint response, plus
SAP/SAF-T: `SapExportConfig`, `SaftExportConfig`, `ChartOfAccountsMeta`,
`SAP_DEFAULT_TABLES` (8) and `SAP_ALL_TABLES` (28).

### Added — examples

- `examples/sap_export.rs` — generate → download → BKPF↔BSEG FK check
- `examples/saft_export.rs` — PT SAF-T fetch with company metadata

### Changed

- `download_file` endpoint switched from `?file=` query to
  `/v1/jobs/{id}/download/{file}` path-suffix.

### Dependencies

- New: `zip = "2"` (deflate-only). `reqwest` gains the `blocking` feature
  for lazy `JobArchive` fetches.

## [1.0.0] - 2026-04-10

### Added

- **Configs resource** (`client.configs()`) — save, list, get, update, and delete generation configs.
  - `validate()` for pre-flight config validation with errors and warnings.
  - `estimate_cost()` for credit cost estimation with multiplier breakdown.
  - `compose()` for merging config layers.
- **Credits resource** (`client.credits()`) — prepaid credit management.
  - `purchase()` returns a Stripe checkout URL for credit pack purchase.
  - `balance()` returns current prepaid balance with active batches.
  - `history()` returns full batch history including expired batches.
- **Sessions resource** (`client.sessions()`) — multi-period generation sessions.
  - `create()`, `list()`, `extend()`, `generate_next()` for longitudinal data generation.
- **Scenarios resource** (`client.scenarios()`) — what-if analysis.
  - `create()`, `list()`, `run()`, `diff()` for baseline vs. counterfactual comparison.
  - `templates()` lists available scenario graph templates.
- **Notifications resource** (`client.notifications()`) — user notifications.
  - `list()` with optional unread/limit filtering via `ListNotificationsParams`.
  - `mark_read()` to mark notifications as read by IDs or all at once.
- `catalog().list_templates()` — list system templates, optionally filtered by sector.
- `ListConfigsParams` and `ListNotificationsParams` for paginated/filtered list operations.
- New types: `SavedConfig`, `Template`, `GenerationSession`, `GenerateSessionResponse`, `Scenario`, `ScenarioTemplate`, `Notification`, `PrepaidBatch`, `PrepaidBalanceResponse`, `PrepaidHistoryResponse`, `ValidateConfigResponse`, `ValidationIssue`, `EstimateCostResponse`, `MultiplierEntry`, `BalanceInfo`, `ComposeConfigResponse`, `DeletedResponse`, and corresponding request types.

### Changed

- Bumped to 1.0.0 — SDK now covers the full stable VynFi API surface.

## [0.2.0] - 2026-04-08

### Added

- `VynFiError::Forbidden` variant for 403 responses.
- `ErrorBody` now follows RFC 7807 with `error_type`, `title`, `detail`, `status`, `instance`.
- `ErrorBody` variants in `VynFiError` are `Box<ErrorBody>` to reduce enum size.
- `GenerateConfigRequest` for config-based generation (portal-style).
- `QuickJobResponse` type for synchronous generation results.
- `CancelJobResponse` type with credit refund details.
- `jobs().generate_config()` for config-based generation requests.
- `jobs().download_file()` to download a specific file from job output.
- `catalog().list()` now accepts optional `sector` and `search` filters.
- `billing().checkout()` and `billing().portal()` for Stripe session creation.
- `WebhookDetail` type with delivery history on `webhooks().get()`.
- `RevokeKeyResponse` type — `api_keys().revoke()` now returns `{id, status, revoked_at}`.
- `TableUsage` type — `by_table` in daily usage is now a structured array.
- `Column.example_values`, `TableDef.id`, `TableDef.slug`, `SectorSummary.id` fields.
- Integration test suite (`tests/integration.rs`) for all endpoints against a real API.

### Changed

- Sector endpoints moved from `/v1/catalog/sectors` to `/v1/sectors`.
- `Job` fields: `user_id`→`owner_id`, `tables`/`format`/`sector_slug`→`config` (JSON), added `progress` (JSON), `artifacts`, `error_detail`.
- `SubmitJobResponse`: restored `links` and `estimated_duration_seconds`.
- `SectorSummary`: `quality_score` is now `i32`, `popularity` is now `i32`.
- `UsageSummary`: `burn_rate` is now `i64`, `period_days` is now `i32`. Removed `tier`.
- `DailyUsageResponse.by_table` changed from `HashMap<String, i64>` to `Vec<TableUsage>`.
- `ApiKey`/`ApiKeyCreated`: removed `scopes` and `expires_at` fields, added `revoked_at`.
- `CreateApiKeyRequest`: simplified to `name` and `environment` only.
- `usage().summary()` now accepts optional `days` parameter.
- `quality().timeline()` parameter changed from `Option<u32>` to `Option<i64>`.
- `jobs().list()` uses offset/limit pagination instead of cursor-based.
- `GenerateRequest`: `format` is `Option<String>`, `sector_slug` is `Option<String>`.
- `GenerateRequest::new()` now takes a `sector_slug` parameter.
- `Invoice` fields updated to match Stripe format (`number`, `amount_due`, `amount_paid`, `hosted_invoice_url`).
- `Subscription` fields updated (`stripe_price_id` instead of `cancel_at_period_end`).
- `Billing` resource: `payment_method()` returns `serde_json::Value` (raw Stripe object).

### Removed

- `FieldError` struct and `ErrorBody.fields` — not in the API.
- `ErrorBody.request_id` — replaced by `instance`.
- `JobProgress` struct — progress is now `serde_json::Value`.
- `DownloadResponse` type — download returns raw bytes.
- `CreditBalance`, `CreditBatch`, `CreditHistory`, `CreditHistoryBatch`, `PurchaseCreditsRequest`, `PurchaseCreditsResponse` — credits resource removed.
- **Credits resource** (`client.credits()`) — not in the API.

## [0.1.0] - 2026-03-01

### Added

- Initial release with async and blocking clients.
- Resources: jobs, catalog, usage, api_keys, quality, webhooks, billing.
