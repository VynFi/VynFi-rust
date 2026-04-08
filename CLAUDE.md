# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

VynFi Rust SDK (`vynfi` crate) — an async-first HTTP client for the VynFi API with an optional blocking wrapper. Published to crates.io under Apache-2.0. MSRV is **1.83**.

## Build & Test Commands

```bash
cargo build                          # build with default features (rustls-tls)
cargo build --features blocking      # build with blocking client
cargo test --features blocking       # run all tests (includes blocking)
cargo test                           # run tests with default features only
cargo test test_name                  # run a single test by name
cargo fmt --check                    # check formatting
cargo clippy --features blocking -- -D warnings  # lint (CI runs this on stable only)
```

CI matrix tests against both stable Rust and MSRV 1.83.

## Architecture

Single-crate library (not a workspace). Entry point: `src/lib.rs`.

**Core layers:**
- **`client.rs`** — Async `Client` and `ClientBuilder`. Handles HTTP transport, retry logic (exponential backoff on 429/5xx, respects `Retry-After`), error mapping from status codes to `VynFiError` variants, and response deserialization. The `extract_list<T>()` helper handles APIs that return either a raw array or `{"data": [...]}`.
- **`error.rs`** — `VynFiError` enum with typed variants per HTTP status (401→Authentication, 402→InsufficientCredits, 403→Forbidden, 404→NotFound, 422→Validation, 429→RateLimit, 5xx→Server). Error body variants are `Box<ErrorBody>` to keep the enum small. `ErrorBody` follows RFC 7807 with `error_type`, `title`, `detail`, `status`, `request_id`, and `fields` (field-level validation errors).
- **`types.rs`** — All request/response structs with serde derive. No business logic.
- **`blocking.rs`** — Feature-gated (`blocking`). Wraps async client with a single-threaded Tokio runtime.

**Resource modules (`src/resources/`):**
Each resource is a lightweight borrowed-reference struct (`&'a Client`) exposing async methods. Five resources: `jobs`, `catalog`, `usage`, `api_keys`, `credits`.

**Key patterns:**
- Resource structs borrow the client (`&'a Client`) — multiple resource handles can coexist
- Generic request helpers (`request<T>`, `request_with_body<T, B>`, `request_with_params<T>`) with `DeserializeOwned` bounds
- TLS backend is configurable via features: `rustls-tls` (default) or `native-tls`

## Tests

Integration tests live in `tests/test_client.rs` using `mockito` for HTTP mocking. All tests use `#[tokio::test]`. Tests cover error mapping, auth headers, response parsing for each resource type.

## Feature Flags

- `rustls-tls` (default) — Rust-native TLS
- `native-tls` — Platform TLS
- `blocking` — Synchronous client wrapper (adds `blocking` module)
