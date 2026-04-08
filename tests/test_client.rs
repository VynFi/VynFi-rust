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
        .mock("GET", "/v1/usage")
        .match_header("authorization", "Bearer vf_test_secret42")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "balance": 1000,
                "total_used": 500,
                "total_reserved": 100,
                "total_refunded": 0,
                "burn_rate": 16.7,
                "period_days": 30,
                "tier": "free"
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_secret42")
        .base_url(server.url())
        .build()
        .unwrap();

    let _ = client.usage().summary().await.unwrap();
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
        .mock("GET", "/v1/usage")
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

    let err = client.usage().summary().await.unwrap_err();
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
        .mock("GET", "/v1/usage")
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

    let err = client.usage().summary().await.unwrap_err();
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
        .mock("POST", "/v1/credits/purchase")
        .with_status(403)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "type": "https://api.vynfi.com/errors/forbidden",
                "title": "Forbidden",
                "status": 403,
                "detail": "Pre-paid packs are only available on the free tier"
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .max_retries(0)
        .build()
        .unwrap();

    let req = vynfi::PurchaseCreditsRequest {
        pack: "starter".to_string(),
    };
    let err = client.credits().purchase(&req).await.unwrap_err();
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
                "object": "job",
                "id": "job_abc123",
                "status": "queued",
                "credits_reserved": 250,
                "message": "Job queued for processing. Poll GET /v1/jobs/job_abc123 for status."
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let req = GenerateRequest {
        tables: vec![TableSpec {
            name: "transactions".to_string(),
            rows: 1000,
        }],
        format: Some("csv".to_string()),
        sector_slug: "banking".to_string(),
    };

    let resp = client.jobs().generate(&req).await.unwrap();
    assert_eq!(resp.id, "job_abc123");
    assert_eq!(resp.status, "queued");
    assert_eq!(resp.credits_reserved, 250);
    assert_eq!(resp.object.as_deref(), Some("job"));
    assert!(resp.message.is_some());

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 8. GET /v1/catalog/sectors parses Vec<SectorSummary>
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_list_sectors() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/catalog/sectors")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "data": [
                    {
                        "slug": "retail",
                        "name": "Retail",
                        "description": "Retail and consumer goods",
                        "icon": "shopping-cart",
                        "multiplier": 1.5,
                        "quality_score": 0.94,
                        "popularity": 95,
                        "table_count": 4
                    }
                ]
            }"#,
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
    assert_eq!(sectors[0].quality_score, Some(0.94));
    assert_eq!(sectors[0].popularity, Some(95));
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
                "user_id": "user-456",
                "status": "completed",
                "tables": null,
                "format": "csv",
                "sector_slug": "banking",
                "rows_requested": 5000,
                "rows_generated": 5000,
                "credits_reserved": 100,
                "credits_used": 95,
                "output_path": "/downloads/job-123.csv",
                "error": null,
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
    assert_eq!(job.user_id.as_deref(), Some("user-456"));
    assert_eq!(job.status, "completed");
    assert_eq!(job.format, "csv");
    assert_eq!(job.rows_requested, Some(5000));
    assert_eq!(job.rows_generated, Some(5000));
    assert_eq!(job.credits_reserved, Some(100));
    assert_eq!(job.credits_used, Some(95));
    assert_eq!(job.sector_slug, "banking");
    assert_eq!(job.output_path.as_deref(), Some("/downloads/job-123.csv"));
    assert!(job.error.is_none());
    assert!(job.started_at.is_some());

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 10. GET /v1/usage parses UsageSummary
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_usage_summary() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/usage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "balance": 7500,
                "total_used": 2300,
                "total_reserved": 200,
                "total_refunded": 50,
                "burn_rate": 76.7,
                "period_days": 30,
                "tier": "free"
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let summary = client.usage().summary().await.unwrap();
    assert_eq!(summary.balance, 7500);
    assert_eq!(summary.total_used, 2300);
    assert_eq!(summary.total_reserved, 200);
    assert_eq!(summary.total_refunded, 50);
    assert!((summary.burn_rate - 76.7).abs() < f64::EPSILON);
    assert_eq!(summary.period_days, 30);
    assert_eq!(summary.tier, "free");

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
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "id": "key_001",
                "name": "CI Pipeline Key",
                "key": "vf_live_abc123def456ghi789jkl012mno345",
                "prefix": "vf_live_abc1",
                "environment": "live",
                "scopes": ["generate", "catalog:read"],
                "expires_at": "2027-03-01T00:00:00Z",
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
        scopes: Some(vec!["generate".to_string(), "catalog:read".to_string()]),
        environment: Some("live".to_string()),
        expires_in_days: Some(365),
    };

    let created = client.api_keys().create(&req).await.unwrap();
    assert_eq!(created.id, "key_001");
    assert_eq!(created.name, "CI Pipeline Key");
    assert_eq!(created.key, "vf_live_abc123def456ghi789jkl012mno345");
    assert_eq!(created.prefix, "vf_live_abc1");
    assert_eq!(created.environment, "live");
    assert_eq!(created.scopes, vec!["generate", "catalog:read"]);
    assert!(created.expires_at.is_some());
    assert!(created.created_at.is_some());

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 12. GET /v1/jobs/:id/download parses DownloadResponse
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_download_job() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/jobs/job-123/download")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "object": "download",
                "url": "https://blob.azure.com/output.json?sas=token",
                "expires_in": 3600
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let dl = client.jobs().download("job-123").await.unwrap();
    assert_eq!(dl.object.as_deref(), Some("download"));
    assert_eq!(dl.url, "https://blob.azure.com/output.json?sas=token");
    assert_eq!(dl.expires_in, 3600);

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 13. GET /v1/credits/balance parses CreditBalance
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_credits_balance() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/credits/balance")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "total_prepaid_credits": 50000,
                "batches": [
                    {
                        "batch_id": "batch-001",
                        "pack": "starter",
                        "credits_remaining": 50000,
                        "credits_purchased": 50000,
                        "expires_at": "2027-04-08T00:00:00Z",
                        "status": "active"
                    }
                ]
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let balance = client.credits().balance().await.unwrap();
    assert_eq!(balance.total_prepaid_credits, 50000);
    assert_eq!(balance.batches.len(), 1);
    assert_eq!(balance.batches[0].batch_id, "batch-001");
    assert_eq!(balance.batches[0].pack, "starter");
    assert_eq!(balance.batches[0].credits_remaining, 50000);
    assert_eq!(balance.batches[0].status, "active");

    mock.assert_async().await;
}
