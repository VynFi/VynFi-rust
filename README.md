# VynFi Rust SDK

Official Rust client for the [VynFi](https://vynfi.com) synthetic financial data API.

## Installation

```toml
[dependencies]
vynfi = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

For the blocking client:
```toml
[dependencies]
vynfi = { version = "1", features = ["blocking"] }
```

## Quick Start

```rust
use vynfi::{Client, GenerateRequest, TableSpec};

#[tokio::main]
async fn main() -> Result<(), vynfi::VynFiError> {
    let client = Client::builder("vf_live_...").build()?;

    // Browse catalog and templates
    let sectors = client.catalog().list_sectors().await?;
    let templates = client.catalog().list_templates(None).await?;

    // Validate and estimate cost before generating
    use vynfi::{ValidateConfigRequest, EstimateCostRequest};
    let config = serde_json::json!({"rows": 1000, "sector": "retail"});

    let validation = client.configs().validate(&ValidateConfigRequest {
        config: config.clone(), partial: None, step: None,
    }).await?;
    println!("Valid: {}", validation.valid);

    let estimate = client.configs().estimate_cost(&EstimateCostRequest {
        config: config.clone(),
    }).await?;
    println!("Cost: {} credits", estimate.total_credits);

    // Generate synthetic data
    let job = client.jobs().generate(&GenerateRequest::new(
        vec![TableSpec { name: "transactions".into(), rows: 1000, base_rate: None }],
        "retail",
    )).await?;
    println!("Job submitted: {}", job.id);

    // Check usage
    let usage = client.usage().summary(None).await?;
    println!("Balance: {} credits", usage.balance);

    Ok(())
}
```

## Blocking Client

```rust
use vynfi::blocking::Client;

fn main() -> Result<(), vynfi::VynFiError> {
    let client = Client::builder("vf_live_...").build()?;
    let sectors = client.catalog().list_sectors()?;
    Ok(())
}
```

## Resources

| Resource | Methods |
|----------|---------|
| `client.jobs()` | `generate`, `generate_config`, `generate_quick`, `list`, `get`, `cancel`, `stream`, `download`, `download_file` |
| `client.catalog()` | `list_sectors`, `get_sector`, `list`, `get_fingerprint`, `list_templates` |
| `client.configs()` | `create`, `list`, `get`, `update`, `delete`, `validate`, `estimate_cost`, `compose` |
| `client.credits()` | `purchase`, `balance`, `history` |
| `client.sessions()` | `list`, `create`, `extend`, `generate_next` |
| `client.scenarios()` | `list`, `create`, `run`, `diff`, `templates` |
| `client.usage()` | `summary`, `daily` |
| `client.api_keys()` | `create`, `list`, `get`, `update`, `revoke` |
| `client.quality()` | `scores`, `timeline` |
| `client.webhooks()` | `create`, `list`, `get`, `update`, `delete`, `test` |
| `client.billing()` | `subscription`, `checkout`, `portal`, `invoices`, `payment_method` |
| `client.notifications()` | `list`, `mark_read` |

## Error Handling

All methods return `Result<T, VynFiError>`. Match on error variants:

```rust
use vynfi::VynFiError;

match client.jobs().get("bad-id").await {
    Ok(job) => println!("Got job: {}", job.id),
    Err(VynFiError::NotFound(_)) => println!("Job not found"),
    Err(VynFiError::RateLimit(_)) => println!("Rate limited, retry later"),
    Err(VynFiError::InsufficientCredits(_)) => println!("Not enough credits"),
    Err(VynFiError::Forbidden(_)) => println!("Forbidden"),
    Err(e) => eprintln!("Error: {e}"),
}
```

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `rustls-tls` | Yes | Use rustls for TLS |
| `native-tls` | No | Use platform-native TLS |
| `blocking` | No | Enable synchronous client |

## License

Apache-2.0
