//! Integration tests against a real VynFi API.
//!
//! These tests are `#[ignore]`d by default so `cargo test` skips them.
//!
//! Run them with:
//!
//! ```sh
//! VYNFI_API_KEY=vf_live_... cargo test --test integration -- --ignored
//! VYNFI_API_KEY=vf_live_... VYNFI_BASE_URL=http://localhost:3001 cargo test --test integration -- --ignored
//! ```

use vynfi::{
    Client, CreateApiKeyRequest, GenerateRequest, ListJobsParams, TableSpec, UpdateApiKeyRequest,
    VynFiError,
};

/// Build a client from environment variables.
fn client() -> Client {
    let api_key = std::env::var("VYNFI_API_KEY").expect("VYNFI_API_KEY must be set");
    let mut builder = Client::builder(api_key);
    if let Ok(url) = std::env::var("VYNFI_BASE_URL") {
        builder = builder.base_url(url);
    }
    builder.build().expect("failed to build client")
}

// ===========================================================================
// Catalog
// ===========================================================================

#[tokio::test]
#[ignore]
async fn catalog_list_sectors() {
    let c = client();
    let sectors = c.catalog().list_sectors().await.unwrap();
    assert!(!sectors.is_empty(), "expected at least one sector");
    // Every sector must have a slug and table_count > 0
    for s in &sectors {
        assert!(!s.slug.is_empty());
        assert!(s.table_count > 0, "sector {} has 0 tables", s.slug);
    }
}

#[tokio::test]
#[ignore]
async fn catalog_get_sector_retail() {
    let c = client();
    let sector = c.catalog().get_sector("retail").await.unwrap();
    assert_eq!(sector.slug, "retail");
    assert!(!sector.tables.is_empty(), "retail should have tables");
    // Check a known table exists
    let names: Vec<&str> = sector.tables.iter().map(|t| t.name.as_str()).collect();
    assert!(
        names.contains(&"journal_entries"),
        "retail should include journal_entries, got: {names:?}"
    );
}

#[tokio::test]
#[ignore]
async fn catalog_get_sector_not_found() {
    let c = client();
    let err = c
        .catalog()
        .get_sector("nonexistent-sector-slug")
        .await
        .unwrap_err();
    assert!(
        matches!(err, VynFiError::NotFound(_)),
        "expected NotFound, got: {err:?}"
    );
}

// ===========================================================================
// Usage
// ===========================================================================

#[tokio::test]
#[ignore]
async fn usage_summary() {
    let c = client();
    let summary = c.usage().summary().await.unwrap();
    // Balance can be zero, but the fields should parse correctly
    assert!(summary.period_days > 0);
    assert!(!summary.tier.is_empty());
}

#[tokio::test]
#[ignore]
async fn usage_daily() {
    let c = client();
    let resp = c.usage().daily(Some(7)).await.unwrap();
    // daily may be empty if no usage, but the struct should parse
    assert!(resp.daily.len() <= 7);
}

// ===========================================================================
// Jobs — list & get (read-only, no credits spent)
// ===========================================================================

#[tokio::test]
#[ignore]
async fn jobs_list() {
    let c = client();
    let list = c
        .jobs()
        .list(&ListJobsParams {
            limit: Some(5),
            ..Default::default()
        })
        .await
        .unwrap();
    // May be empty on a fresh account, but should parse
    for job in &list.jobs {
        assert!(!job.id.is_empty());
        assert!(!job.status.is_empty());
    }
}

#[tokio::test]
#[ignore]
async fn jobs_get_not_found() {
    let c = client();
    let err = c
        .jobs()
        .get("00000000-0000-0000-0000-000000000000")
        .await
        .unwrap_err();
    assert!(
        matches!(err, VynFiError::NotFound(_)),
        "expected NotFound, got: {err:?}"
    );
}

// ===========================================================================
// Jobs — generate quick (spends credits)
// ===========================================================================

#[tokio::test]
#[ignore]
async fn jobs_generate_quick_and_download() {
    let c = client();

    let req = GenerateRequest::new(
        vec![TableSpec {
            name: "journal_entries".to_string(),
            rows: 10,
        }],
        "retail",
    );

    let job = c.jobs().generate_quick(&req).await.unwrap();
    assert_eq!(job.status, "completed");
    assert!(job.rows_generated.unwrap_or(0) > 0);
    assert!(job.credits_used.unwrap_or(0) > 0);

    // Verify we can fetch the same job by ID
    let fetched = c.jobs().get(&job.id).await.unwrap();
    assert_eq!(fetched.id, job.id);
    assert_eq!(fetched.status, "completed");

    // Download the output URL
    let dl = c.jobs().download(&job.id).await.unwrap();
    assert!(!dl.url.is_empty());
    assert!(dl.expires_in > 0);
}

#[tokio::test]
#[ignore]
async fn jobs_generate_async() {
    let c = client();

    let req = GenerateRequest::new(
        vec![TableSpec {
            name: "journal_entries".to_string(),
            rows: 10,
        }],
        "retail",
    );

    let resp = c.jobs().generate(&req).await.unwrap();
    assert!(!resp.id.is_empty());
    assert!(!resp.status.is_empty());
    assert!(resp.credits_reserved > 0);
}

#[tokio::test]
#[ignore]
async fn jobs_generate_validation_error() {
    let c = client();

    // Empty tables should fail validation
    let req = GenerateRequest {
        tables: vec![],
        format: None,
        sector_slug: "retail".to_string(),
    };

    let err = c.jobs().generate_quick(&req).await.unwrap_err();
    assert!(
        matches!(err, VynFiError::Validation(_)),
        "expected Validation, got: {err:?}"
    );
}

// ===========================================================================
// API Keys — full CRUD lifecycle
// ===========================================================================

#[tokio::test]
#[ignore]
async fn api_keys_lifecycle() {
    let c = client();

    // Create
    let created = c
        .api_keys()
        .create(&CreateApiKeyRequest {
            name: "integration-test-key".to_string(),
            scopes: Some(vec!["catalog:read".to_string()]),
            environment: Some("test".to_string()),
            expires_in_days: Some(1),
        })
        .await
        .unwrap();

    assert!(!created.id.is_empty());
    assert!(!created.key.is_empty(), "full secret should be returned");
    assert!(
        created.key.starts_with("vf_test_"),
        "test env key should have vf_test_ prefix, got: {}",
        created.key
    );
    assert_eq!(created.name, "integration-test-key");
    assert_eq!(created.environment, "test");

    let key_id = created.id.clone();

    // List — the new key should appear
    let keys = c.api_keys().list().await.unwrap();
    assert!(
        keys.iter().any(|k| k.id == key_id),
        "newly created key should appear in list"
    );

    // Get
    let fetched = c.api_keys().get(&key_id).await.unwrap();
    assert_eq!(fetched.id, key_id);
    assert_eq!(fetched.name, "integration-test-key");

    // Update
    let updated = c
        .api_keys()
        .update(
            &key_id,
            &UpdateApiKeyRequest {
                name: Some("integration-test-key-updated".to_string()),
                scopes: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.name, "integration-test-key-updated");

    // Revoke (cleanup)
    c.api_keys().revoke(&key_id).await.unwrap();

    // Verify it's gone / revoked
    let keys_after = c.api_keys().list().await.unwrap();
    let revoked = keys_after.iter().find(|k| k.id == key_id);
    // Either the key is absent or its status is "revoked"
    if let Some(k) = revoked {
        assert_eq!(k.status, "revoked");
    }
}

// ===========================================================================
// Credits
// ===========================================================================

#[tokio::test]
#[ignore]
async fn credits_balance() {
    let c = client();
    let balance = c.credits().balance().await.unwrap();
    // total_prepaid_credits may be 0, but the struct should parse
    assert!(balance.total_prepaid_credits >= 0);
}

#[tokio::test]
#[ignore]
async fn credits_history() {
    let c = client();
    let history = c.credits().history().await.unwrap();
    // batches may be empty on a fresh account
    for batch in &history.batches {
        assert!(!batch.batch_id.is_empty());
        assert!(!batch.pack.is_empty());
    }
}

// Note: credits_purchase is NOT tested — it would create a real Stripe
// checkout session and potentially charge money.
