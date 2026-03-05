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
                "burn_rate": 16.7,
                "period_days": 30
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
        .with_body(r#"{"detail": "not found", "message": "not found", "status": 404}"#)
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
        .with_body(r#"{"detail": "rate limited", "message": "rate limited", "status": 429}"#)
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
        .mock("GET", "/v1/usage/summary")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"detail": "internal error", "message": "internal error", "status": 500}"#)
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
// 6. POST /v1/generate parses SubmitJobResponse
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_generate_job() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/generate")
        .with_status(200)
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
                    "cancel": "/v1/jobs/job_abc123",
                    "download": "/v1/jobs/job_abc123/download"
                }
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
        format: "csv".to_string(),
        sector_slug: "banking".to_string(),
    };

    let resp = client.jobs().generate(&req).await.unwrap();
    assert_eq!(resp.id, "job_abc123");
    assert_eq!(resp.status, "queued");
    assert_eq!(resp.credits_reserved, 250);
    assert_eq!(resp.estimated_duration_seconds, 12);

    let links = resp.links.expect("links should be present");
    assert_eq!(links.self_link, "/v1/jobs/job_abc123");
    assert_eq!(links.stream, "/v1/jobs/job_abc123/stream");
    assert_eq!(links.cancel, "/v1/jobs/job_abc123");
    assert_eq!(links.download, "/v1/jobs/job_abc123/download");

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 7. GET /v1/catalog/sectors parses Vec<SectorSummary>
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
                        "slug": "banking",
                        "name": "Banking",
                        "description": "",
                        "icon": "",
                        "table_count": 3
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
    assert_eq!(sectors[0].slug, "banking");
    assert_eq!(sectors[0].name, "Banking");
    assert_eq!(sectors[0].table_count, 3);

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 8. GET /v1/jobs/:id parses Job
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
                "status": "completed",
                "tables": null,
                "format": "csv",
                "credits_reserved": 100,
                "credits_used": 95,
                "sector_slug": "banking",
                "progress": {
                    "percent": 100,
                    "rows_generated": 5000,
                    "rows_total": 5000
                },
                "output_path": "/downloads/job-123.csv",
                "error": null,
                "created_at": "2026-03-01T10:00:00Z",
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
    assert_eq!(job.status, "completed");
    assert_eq!(job.format, "csv");
    assert_eq!(job.credits_reserved, Some(100));
    assert_eq!(job.credits_used, Some(95));
    assert_eq!(job.sector_slug, "banking");
    assert_eq!(job.output_path.as_deref(), Some("/downloads/job-123.csv"));
    assert!(job.error.is_none());

    let progress = job.progress.expect("progress should be present");
    assert_eq!(progress.percent, 100);
    assert_eq!(progress.rows_generated, 5000);
    assert_eq!(progress.rows_total, 5000);

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 9. GET /v1/usage/summary parses UsageSummary
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
                "burn_rate": 76.7,
                "period_days": 30
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

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 10. POST /v1/api-keys parses ApiKeyCreated
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
        expires_in_days: Some(365),
    };

    let created = client.api_keys().create(&req).await.unwrap();
    assert_eq!(created.id, "key_001");
    assert_eq!(created.name, "CI Pipeline Key");
    assert_eq!(created.key, "vf_live_abc123def456ghi789jkl012mno345");
    assert_eq!(created.prefix, "vf_live_abc1");
    assert_eq!(created.scopes, vec!["generate", "catalog:read"]);
    assert!(created.expires_at.is_some());
    assert!(created.created_at.is_some());

    mock.assert_async().await;
}
