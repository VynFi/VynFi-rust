//! VynFi SAF-T export — PT jurisdiction, OECD 1.04_01.

use std::time::Duration;

use serde_json::json;
use vynfi::{Client, SaftExportConfig};

#[tokio::main]
async fn main() -> Result<(), vynfi::VynFiError> {
    let api_key = std::env::var("VYNFI_API_KEY").expect("VYNFI_API_KEY not set");
    let client = Client::builder(api_key)
        .timeout(Duration::from_secs(180))
        .build()?;

    let mut saft = SaftExportConfig::new("pt");
    saft.company_tax_id = Some("500000000".into());
    saft.company_name = Some("ACME Retail SA".into());

    let config = json!({
        "sector": "retail",
        "country": "PT",
        "accountingFramework": "ifrs",
        "rows": 300,
        "companies": 1,
        "periods": 1,
        "processModels": ["o2c", "p2p"],
        "exportFormat": "saft",
        "output": { "saft": saft },
    });
    let req = vynfi::GenerateConfigRequest {
        config,
        config_id: None,
    };

    println!("Submitting SAF-T job …");
    let job = client.jobs().generate_config(&req).await?;
    let done = client
        .jobs()
        .wait(&job.id, Duration::from_secs(5), Duration::from_secs(420))
        .await?;
    println!("  {}: {}", job.id, done.status);
    if done.status != "completed" {
        return Ok(());
    }

    let mut archive = client.jobs().download_archive(&job.id).await?;
    let xml = archive.saft_file("pt").map_err(vynfi::VynFiError::Config)?;
    let head = std::str::from_utf8(&xml[..xml.len().min(180)]).unwrap_or("(non-utf8)");
    println!("  saft_pt.xml: {} bytes", xml.len());
    println!("  head: {}", head);
    std::fs::write("/tmp/saft_pt.xml", &xml).ok();
    println!("  saved to /tmp/saft_pt.xml");
    Ok(())
}
