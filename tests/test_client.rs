use vynfi::{
    CreateApiKeyRequest, CreateConfigRequest, CreateScenarioRequest, CreateSessionRequest,
    EstimateCostRequest, GenerateRequest, ListConfigsParams, ListNotificationsParams,
    MarkReadRequest, PurchaseCreditsRequest, TableSpec, ValidateConfigRequest, VynFiError,
};

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

// ---------------------------------------------------------------------------
// 14. POST /v1/configs creates a saved config
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_create_config() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/configs")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "id": "cfg-001",
                "ownerId": "user-1",
                "name": "Retail Standard",
                "description": "Standard retail config",
                "config": {"rows": 1000, "sector": "retail"},
                "sourceTemplateId": null,
                "version": 1,
                "visibility": "private",
                "tags": ["retail", "standard"],
                "lastUsedAt": null,
                "createdAt": "2026-04-01T10:00:00Z",
                "updatedAt": "2026-04-01T10:00:00Z",
                "schemaVersion": 1
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let req = CreateConfigRequest {
        name: "Retail Standard".to_string(),
        description: Some("Standard retail config".to_string()),
        config: serde_json::json!({"rows": 1000, "sector": "retail"}),
        source_template_id: None,
        visibility: Some("private".to_string()),
        tags: Some(vec!["retail".to_string(), "standard".to_string()]),
    };

    let cfg = client.configs().create(&req).await.unwrap();
    assert_eq!(cfg.id, "cfg-001");
    assert_eq!(cfg.name, "Retail Standard");
    assert_eq!(cfg.visibility, "private");
    assert_eq!(cfg.tags, vec!["retail", "standard"]);
    assert_eq!(cfg.version, 1);

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 15. GET /v1/configs lists saved configs
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_list_configs() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/configs")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "configs": [{
                    "id": "cfg-001",
                    "ownerId": "user-1",
                    "name": "My Config",
                    "description": "",
                    "config": {},
                    "sourceTemplateId": null,
                    "version": 1,
                    "visibility": "private",
                    "tags": [],
                    "lastUsedAt": null,
                    "createdAt": "2026-04-01T10:00:00Z",
                    "updatedAt": "2026-04-01T10:00:00Z",
                    "schemaVersion": null
                }]
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let configs = client
        .configs()
        .list(&ListConfigsParams::default())
        .await
        .unwrap();
    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].id, "cfg-001");
    assert_eq!(configs[0].name, "My Config");

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 16. POST /v1/config/validate validates config
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_validate_config() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/config/validate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "valid": false,
                "errors": [{
                    "field": "rows",
                    "code": "required",
                    "message": "rows is required",
                    "fix": null
                }],
                "warnings": []
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let req = ValidateConfigRequest {
        config: serde_json::json!({"sector": "retail"}),
        partial: None,
        step: None,
    };

    let resp = client.configs().validate(&req).await.unwrap();
    assert!(!resp.valid);
    assert_eq!(resp.errors.len(), 1);
    assert_eq!(resp.errors[0].field, "rows");
    assert_eq!(resp.errors[0].code, "required");
    assert!(resp.warnings.is_empty());

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 17. POST /v1/config/estimate-cost estimates credit cost
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_estimate_cost() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/config/estimate-cost")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "baseCredits": 100,
                "multipliers": [
                    {"source": "sector", "factor": 1.5, "label": "Retail sector"}
                ],
                "totalCredits": 150,
                "cappedAt": null,
                "balance": {
                    "current": 10000,
                    "afterJob": 9850,
                    "status": "sufficient"
                }
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let req = EstimateCostRequest {
        config: serde_json::json!({"rows": 1000, "sector": "retail"}),
    };

    let resp = client.configs().estimate_cost(&req).await.unwrap();
    assert_eq!(resp.base_credits, 100);
    assert_eq!(resp.total_credits, 150);
    assert_eq!(resp.multipliers.len(), 1);
    assert_eq!(resp.multipliers[0].factor, 1.5);
    assert_eq!(resp.balance.current, 10000);
    assert_eq!(resp.balance.after_job, 9850);
    assert_eq!(resp.balance.status, "sufficient");

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 18. GET /v1/credits/balance parses PrepaidBalanceResponse
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
                "batches": [{
                    "id": "batch-001",
                    "ownerId": "user-1",
                    "pack": "50k",
                    "creditsPurchased": 50000,
                    "creditsRemaining": 48000,
                    "creditsForfeited": 0,
                    "status": "active",
                    "purchasedAt": "2026-03-01T10:00:00Z",
                    "expiresAt": "2027-03-01T10:00:00Z",
                    "createdAt": "2026-03-01T10:00:00Z"
                }]
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let resp = client.credits().balance().await.unwrap();
    assert_eq!(resp.total_prepaid_credits, 50000);
    assert_eq!(resp.batches.len(), 1);
    assert_eq!(resp.batches[0].pack, "50k");
    assert_eq!(resp.batches[0].credits_purchased, 50000);
    assert_eq!(resp.batches[0].credits_remaining, 48000);
    assert_eq!(resp.batches[0].status, "active");

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 19. POST /v1/credits/purchase returns checkout URL
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_purchase_credits() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/credits/purchase")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"checkout_url": "https://checkout.stripe.com/session_xyz"}"#)
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let req = PurchaseCreditsRequest {
        pack: "50k".to_string(),
    };

    let resp = client.credits().purchase(&req).await.unwrap();
    assert_eq!(resp.checkout_url, "https://checkout.stripe.com/session_xyz");

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 20. POST /v1/sessions creates a session
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_create_session() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/sessions")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "id": "sess-001",
                "name": "FY2026 Quarterly",
                "status": "created",
                "fiscalYearStart": "2026-01-01",
                "periodLengthMonths": 3,
                "periodsTotal": 4,
                "periodsGenerated": 0,
                "periods": [],
                "balanceSnapshot": null,
                "generationConfig": {"sector": "retail", "rows": 5000},
                "createdAt": "2026-04-01T10:00:00Z",
                "updatedAt": "2026-04-01T10:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let req = CreateSessionRequest {
        name: "FY2026 Quarterly".to_string(),
        fiscal_year_start: "2026-01-01".to_string(),
        period_length_months: 3,
        periods: 4,
        generation_config: serde_json::json!({"sector": "retail", "rows": 5000}),
    };

    let sess = client.sessions().create(&req).await.unwrap();
    assert_eq!(sess.id, "sess-001");
    assert_eq!(sess.name, "FY2026 Quarterly");
    assert_eq!(sess.status, "created");
    assert_eq!(sess.period_length_months, 3);
    assert_eq!(sess.periods_total, 4);
    assert_eq!(sess.periods_generated, 0);

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 21. GET /v1/sessions lists sessions
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_list_sessions() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/sessions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"[{
                "id": "sess-001",
                "name": "FY2026",
                "status": "created",
                "fiscalYearStart": "2026-01-01",
                "periodLengthMonths": 3,
                "periodsTotal": 4,
                "periodsGenerated": 0,
                "periods": [],
                "balanceSnapshot": null,
                "generationConfig": {},
                "createdAt": "2026-04-01T10:00:00Z",
                "updatedAt": "2026-04-01T10:00:00Z"
            }]"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let sessions = client.sessions().list().await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, "sess-001");
    assert_eq!(sessions[0].name, "FY2026");

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 22. POST /v1/scenarios creates a scenario
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_create_scenario() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/scenarios")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "id": "scn-001",
                "name": "Revenue Fraud Impact",
                "templateId": "tpl-001",
                "status": "created",
                "interventions": {"fraudRate": 0.05},
                "generationConfig": {"sector": "retail", "rows": 1000},
                "baselineJobId": null,
                "counterfactualJobId": null,
                "diff": null,
                "createdAt": "2026-04-01T10:00:00Z",
                "updatedAt": "2026-04-01T10:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let req = CreateScenarioRequest {
        name: "Revenue Fraud Impact".to_string(),
        template_id: "tpl-001".to_string(),
        interventions: serde_json::json!({"fraudRate": 0.05}),
        generation_config: serde_json::json!({"sector": "retail", "rows": 1000}),
    };

    let scn = client.scenarios().create(&req).await.unwrap();
    assert_eq!(scn.id, "scn-001");
    assert_eq!(scn.name, "Revenue Fraud Impact");
    assert_eq!(scn.status, "created");
    assert!(scn.baseline_job_id.is_none());
    assert!(scn.counterfactual_job_id.is_none());

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 23. GET /v1/notifications lists notifications
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_list_notifications() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/notifications")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"[{
                "id": "notif-001",
                "user_id": "user-1",
                "type": "job.completed",
                "title": "Job completed",
                "message": "Your generation job has finished",
                "link": "/jobs/job-123",
                "read": false,
                "created_at": "2026-04-01T10:00:00Z"
            }]"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let notifs = client
        .notifications()
        .list(&ListNotificationsParams::default())
        .await
        .unwrap();
    assert_eq!(notifs.len(), 1);
    assert_eq!(notifs[0].id, "notif-001");
    assert_eq!(notifs[0].notification_type, "job.completed");
    assert_eq!(notifs[0].title, "Job completed");
    assert!(!notifs[0].read);
    assert_eq!(notifs[0].link.as_deref(), Some("/jobs/job-123"));

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 24. POST /v1/notifications/read marks notifications as read
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_mark_notifications_read() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/notifications/read")
        .with_status(204)
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let req = MarkReadRequest {
        ids: None,
        all: Some(true),
    };

    client.notifications().mark_read(&req).await.unwrap();
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 25. GET /v1/templates lists templates
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_list_templates() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/v1/templates")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "templates": [{
                    "id": "tpl-001",
                    "slug": "retail-standard",
                    "name": "Retail Standard",
                    "description": "Standard retail data generation",
                    "sector": "retail",
                    "country": "US",
                    "framework": "GAAP",
                    "config": {"rows": 1000},
                    "minTier": "free",
                    "sortOrder": 1
                }]
            }"#,
        )
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let templates = client.catalog().list_templates(None).await.unwrap();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].id, "tpl-001");
    assert_eq!(templates[0].slug, "retail-standard");
    assert_eq!(templates[0].name, "Retail Standard");
    assert_eq!(templates[0].sector, "retail");
    assert_eq!(templates[0].min_tier, "free");
    assert_eq!(templates[0].sort_order, 1);

    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// 26. DELETE /v1/configs/:id deletes a config
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_delete_config() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("DELETE", "/v1/configs/cfg-001")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"deleted": true}"#)
        .create_async()
        .await;

    let client = vynfi::Client::builder("vf_test_key")
        .base_url(server.url())
        .build()
        .unwrap();

    let resp = client.configs().delete("cfg-001").await.unwrap();
    assert!(resp.deleted);

    mock.assert_async().await;
}
