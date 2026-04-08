use vynfi::{CreateApiKeyRequest, GenerateRequest, TableSpec, VynFiError};

// ---------------------------------------------------------------------------
// 1. Empty API key returns Config error
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_requires_api_key() {
    let result = vynfi::Client::builder("").build();
    assert!(result.is_err(), "expected Err, got Ok");
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("expected Err"),
    };
    assert!(
        matches!(err, VynFiError::Config(_)),
        "expected VynFiError::Config, got: {err:?}"
    );
}

// ---------------------------------------------------------------------------
// 2. Authorization header is sent as "Bearer <key>"
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_auth_header() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/usage/summary")
        .match_header("authorization", "Bearer vf_test_secret42")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "balance": 1000,
                "total_used": 500,
                "total_reserved": 100,
                "total_refunded": 0,
                "burn_rate": 17,
                "period_days": 30
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_secret42")
        .base_url(server.url())
        .build()
        .unwrap();

    let _ = client.usage().summary(None).await.unwrap();
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 3. 404 maps to VynFiError::NotFound
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_404_returns_not_found() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/jobs/nonexistent")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "type": "https://api.vynfi.com/errors/not-found",
                "title": "Not Found",
                "status": 404,
                "detail": "Job not found"
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .max_retries(0)
        .build()
        .unwrap();

    let err = client.jobs().get("nonexistent").await.unwrap_err();
    assert!(
        matches!(err, VynFiError::NotFound(_)),
        "expected VynFiError::NotFound, got: {err:?}"
    );
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 4. 429 maps to VynFiError::RateLimit
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_429_returns_rate_limit() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/usage/summary")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "type": "https://api.vynfi.com/errors/rate-limit-exceeded",
                "title": "Rate Limit Exceeded",
                "status": 429,
                "detail": "rate limited"
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .max_retries(0)
        .build()
        .unwrap();

    let err = client.usage().summary(None).await.unwrap_err();
    assert!(
        matches!(err, VynFiError::RateLimit(_)),
        "expected VynFiError::RateLimit, got: {err:?}"
    );
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 5. 500 maps to VynFiError::Server
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_500_returns_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/usage/summary")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "type": "https://api.vynfi.com/errors/internal-error",
                "title": "Internal Server Error",
                "status": 500,
                "detail": "internal error"
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .max_retries(0)
        .build()
        .unwrap();

    let err = client.usage().summary(None).await.unwrap_err();
    assert!(
        matches!(err, VynFiError::Server(_)),
        "expected VynFiError::Server, got: {err:?}"
    );
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 6. 403 maps to VynFiError::Forbidden
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_403_returns_forbidden() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/generate")
        .with_status(403)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "type": "https://api.vynfi.com/errors/forbidden",
                "title": "Forbidden",
                "status": 403,
                "detail": "Insufficient permissions"
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .max_retries(0)
        .build()
        .unwrap();

    let req = GenerateRequest::new(
        vec![TableSpec {
            name: "journal_entries".to_string(),
            rows: 10,
            base_rate: None,
        }],
        "retail",
    );
    let err = client.jobs().generate(&req).await.unwrap_err();
    assert!(
        matches!(err, VynFiError::Forbidden(_)),
        "expected VynFiError::Forbidden, got: {err:?}"
    );
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 7. POST /v1/generate parses SubmitJobResponse
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_generate_job() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/generate")
        .with_status(202)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "id": "job_abc123",
                "status": "queued",
                "credits_reserved": 250,
                "estimated_duration_seconds": 12,
                "links": {
                    "self": "/v1/jobs/job_abc123",
                    "stream": "/v1/jobs/job_abc123/stream",
                    "cancel": "/v1/jobs/job_abc123"
                }
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let req = GenerateRequest::new(
        vec![TableSpec {
            name: "transactions".to_string(),
            rows: 1000,
            base_rate: None,
        }],
        "retail",
    );

    let resp = client.jobs().generate(&req).await.unwrap();
    assert_eq!(resp.id, "job_abc123");
    assert_eq!(resp.status, "queued");
    assert_eq!(resp.credits_reserved, 250);
    assert_eq!(resp.estimated_duration_seconds, 12);

    let links = resp.links.expect("links should be present");
    assert_eq!(links.self_link, "/v1/jobs/job_abc123");
    assert_eq!(links.stream, "/v1/jobs/job_abc123/stream");
    assert_eq!(links.cancel, "/v1/jobs/job_abc123");

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 8. GET /v1/sectors parses Vec<SectorSummary>
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_list_sectors() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/sectors")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"[
                {
                    "id": "uuid-1",
                    "slug": "retail",
                    "name": "Retail",
                    "description": "Retail and consumer goods",
                    "icon": "shopping-cart",
                    "multiplier": 1.5,
                    "quality_score": 94,
                    "popularity": 95,
                    "table_count": 4
                }
            ]"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let sectors = client.catalog().list_sectors().await.unwrap();
    assert_eq!(sectors.len(), 1);
    assert_eq!(sectors[0].slug, "retail");
    assert_eq!(sectors[0].name, "Retail");
    assert_eq!(sectors[0].multiplier, 1.5);
    assert_eq!(sectors[0].quality_score, 94);
    assert_eq!(sectors[0].popularity, 95);
    assert_eq!(sectors[0].table_count, 4);

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 9. GET /v1/jobs/:id parses Job
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_get_job() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/jobs/job-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "id": "job-123",
                "owner_id": "user-456",
                "status": "completed",
                "config": {"rows": 5000, "sector": "retail"},
                "progress": {"percent": 100, "rows_generated": 5000, "rows_total": 5000},
                "credits_reserved": 100,
                "credits_used": 95,
                "artifacts": null,
                "error_detail": null,
                "created_at": "2026-03-01T10:00:00Z",
                "started_at": "2026-03-01T10:00:05Z",
                "completed_at": "2026-03-01T10:01:00Z"
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let job = client.jobs().get("job-123").await.unwrap();
    assert_eq!(job.id, "job-123");
    assert_eq!(job.owner_id.as_deref(), Some("user-456"));
    assert_eq!(job.status, "completed");
    assert!(job.config.is_some());
    assert_eq!(job.credits_reserved, 100);
    assert_eq!(job.credits_used, Some(95));
    assert!(job.error_detail.is_none());
    assert!(job.started_at.is_some());

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 10. GET /v1/usage/summary parses UsageSummary
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_usage_summary() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/usage/summary")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "balance": 7500,
                "total_used": 2300,
                "total_reserved": 200,
                "total_refunded": 50,
                "burn_rate": 77,
                "period_days": 30
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let summary = client.usage().summary(None).await.unwrap();
    assert_eq!(summary.balance, 7500);
    assert_eq!(summary.total_used, 2300);
    assert_eq!(summary.total_reserved, 200);
    assert_eq!(summary.total_refunded, 50);
    assert_eq!(summary.burn_rate, 77);
    assert_eq!(summary.period_days, 30);

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 11. POST /v1/api-keys parses ApiKeyCreated
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_create_api_key() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/api-keys")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "id": "key_001",
                "name": "CI Pipeline Key",
                "key": "vf_live_abc123def456ghi789jkl012mno345",
                "prefix": "vf_live_abc123de",
                "environment": "live",
                "created_at": "2026-03-01T12:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let req = CreateApiKeyRequest {
        name: "CI Pipeline Key".to_string(),
        environment: Some("live".to_string()),
    };

    let created = client.api_keys().create(&req).await.unwrap();
    assert_eq!(created.id, "key_001");
    assert_eq!(created.name, "CI Pipeline Key");
    assert_eq!(created.key, "vf_live_abc123def456ghi789jkl012mno345");
    assert_eq!(created.prefix, "vf_live_abc123de");
    assert_eq!(created.environment, "live");
    assert!(created.created_at.is_some());

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 12. DELETE /v1/jobs/:id parses CancelJobResponse
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_cancel_job() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("DELETE", "/v1/jobs/job-456")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "id": "job-456",
                "status": "cancelled",
                "credits_reserved": 500,
                "credits_used": 100,
                "credits_refunded": 400,
                "rows_generated": 2000,
                "rows_total": 10000
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let resp = client.jobs().cancel("job-456").await.unwrap();
    assert_eq!(resp.id, "job-456");
    assert_eq!(resp.status, "cancelled");
    assert_eq!(resp.credits_refunded, 400);
    assert_eq!(resp.rows_generated, 2000);

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 13. DELETE /v1/api-keys/:id parses RevokeKeyResponse
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_revoke_api_key() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("DELETE", "/v1/api-keys/key-789")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "id": "key-789",
                "status": "revoked",
                "revoked_at": "2026-04-08T10:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let resp = client.api_keys().revoke("key-789").await.unwrap();
    assert_eq!(resp.id, "key-789");
    assert_eq!(resp.status, "revoked");
    assert!(resp.revoked_at.is_some());

    mock.assert_async().await;
}
