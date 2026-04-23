//! VynFi SAP Integration Pack — generate, download, foreign-key verify.
//!
//! Run with `cargo run --example sap_export` (requires `VYNFI_API_KEY`).

use std::time::Duration;

use serde_json::json;
use vynfi::{Client, SapExportConfig, VynFiError};

#[tokio::main]
async fn main() -> Result<(), VynFiError> {
    let api_key = std::env::var("VYNFI_API_KEY").expect("VYNFI_API_KEY not set");
    let client = Client::builder(api_key)
        .timeout(Duration::from_secs(180))
        .build()?;

    let sap = SapExportConfig {
        dialect: "hana".into(),
        client: "200".into(),
        ledger: "0L".into(),
        source_system: "DATASYNTH".into(),
        local_currency: Some("EUR".into()),
        ..Default::default()
    };

    let config = json!({
        "sector": "retail",
        "country": "DE",
        "accountingFramework": "ifrs",
        "rows": 500,
        "companies": 1,
        "periods": 2,
        "processModels": ["o2c", "p2p"],
        "exportFormat": "sap",
        "output": { "sap": sap },
    });

    let req = vynfi::GenerateConfigRequest {
        config,
        config_id: None,
    };

    println!("Submitting SAP export job …");
    let job = client.jobs().generate_config(&req).await?;
    println!("  job {}", job.id);

    let done = client
        .jobs()
        .wait(&job.id, Duration::from_secs(5), Duration::from_secs(420))
        .await?;
    println!("  status: {}", done.status);
    if done.status != "completed" {
        eprintln!("generation failed: {:?}", done.error_detail);
        return Ok(());
    }

    let mut archive = client.jobs().download_archive(&job.id).await?;
    println!("  backend: {}", archive.backend());

    let tables = archive.sap_tables();
    println!("  {} SAP tables emitted: {:?}", tables.len(), tables);

    // Foreign-key verification: every BSEG.BELNR should resolve to a BKPF.BELNR.
    let bkpf_bytes = archive.sap_table("bkpf").map_err(VynFiError::Config)?;
    let bseg_bytes = archive.sap_table("bseg").map_err(VynFiError::Config)?;
    let bkpf_ids = extract_column(&bkpf_bytes, 2); // MANDT;BUKRS;BELNR;...
    let bseg_ids = extract_column(&bseg_bytes, 2);
    let orphans: Vec<_> = bseg_ids
        .iter()
        .filter(|b| !bkpf_ids.contains(b.as_str()))
        .collect();
    if orphans.is_empty() {
        println!("  ✓ BSEG.BELNR ⊆ BKPF.BELNR (FK integrity holds)");
    } else {
        println!("  ✗ {} BSEG.BELNR rows orphaned", orphans.len());
    }

    Ok(())
}

fn extract_column(raw: &[u8], idx: usize) -> std::collections::HashSet<String> {
    let body = if raw.len() >= 3 && raw[..3] == [0xef, 0xbb, 0xbf] {
        &raw[3..]
    } else {
        raw
    };
    let text = std::str::from_utf8(body).unwrap_or_default();
    text.lines()
        .skip(1)
        .filter_map(|l| l.split(';').nth(idx).map(|s| s.to_owned()))
        .collect()
}
