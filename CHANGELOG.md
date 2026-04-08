# Changelog

All notable changes to this project will be documented in this file.

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
