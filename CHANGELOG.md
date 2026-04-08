# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2026-04-08

### Added

- **Credits resource** (`client.credits()`) — `purchase`, `balance`, `history` for prepaid credit pack management.
- `DownloadResponse` type — `jobs().download()` now returns a presigned URL with expiry instead of raw bytes.
- `VynFiError::Forbidden` variant for 403 responses.
- `FieldError` struct and `ErrorBody.fields` for field-level validation errors.
- `ErrorBody` now includes `error_type`, `title`, `request_id` fields (RFC 7807).
- `Job` fields: `user_id`, `rows_requested`, `rows_generated`, `started_at`.
- `SubmitJobResponse` fields: `object`, `message`.
- `SectorSummary` fields: `multiplier`, `quality_score`, `popularity`.
- `UsageSummary.tier` field.
- `environment` field on `ApiKey`, `ApiKeyCreated`, and `CreateApiKeyRequest`.
- Integration test suite (`tests/integration.rs`) for all endpoints against a real API.

### Changed

- `jobs().download()` returns `DownloadResponse` (JSON with presigned URL) instead of `bytes::Bytes`.
- `usage().summary()` now calls `/v1/usage` (was `/v1/usage/summary`).
- `GenerateRequest.format` changed from `String` to `Option<String>` (server defaults to JSON).
- `GenerateRequest::new()` now takes a `sector_slug` parameter.
- `ErrorBody` variants in `VynFiError` are now `Box<ErrorBody>` to reduce enum size.

### Removed

- **Quality resource** (`client.quality()`) — not in stabilized API.
- **Webhooks resource** (`client.webhooks()`) — not in stabilized API.
- **Billing resource** (`client.billing()`) — not in stabilized API.
- `jobs().cancel()` — no cancel endpoint in API.
- `jobs().stream()` — no SSE streaming endpoint in API.
- `catalog().list()` and `catalog().get_fingerprint()` — no such endpoints in API.
- Types: `JobLinks`, `JobProgress`, `SseEvent`, `CatalogItem`, `Fingerprint`, `QualityScore`, `DailyQuality`, `Webhook`, `WebhookCreated`, `CreateWebhookRequest`, `UpdateWebhookRequest`, `Subscription`, `Invoice`, `PaymentMethod`.
- Dependencies: `reqwest-eventsource`, `futures-core`, `bytes`.

## [0.1.0] - 2026-03-01

### Added

- Initial release with async and blocking clients.
- Resources: jobs, catalog, usage, api_keys, quality, webhooks, billing.
