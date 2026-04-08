# VynFi Rust SDK

Official Rust client for the [VynFi](https://vynfi.com) synthetic financial data API.

## Installation

```toml
[dependencies]
vynfi = "0.2"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

For the blocking client:
```toml
[dependencies]
vynfi = { version = "0.2", features = ["blocking"] }
```

## Quick Start

```rust
use vynfi::{Client, GenerateRequest, TableSpec};

#[tokio::main]
async fn main() -> Result<(), vynfi::VynFiError> {
    let client = Client::builder("vf_live_...").build()?;

    // Generate synthetic data
    let job = client.jobs().generate(&GenerateRequest::new(
        vec![TableSpec { name: "transactions".into(), rows: 1000, base_rate: None }],
        "retail",
    )).await?;
    println!("Job submitted: {}", job.id);

    // Browse catalog
    let sectors = client.catalog().list_sectors().await?;
    for s in sectors {
        println!("{}: {}", s.slug, s.name);
    }

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
| `client.catalog()` | `list_sectors`, `get_sector`, `list`, `get_fingerprint` |
| `client.usage()` | `summary`, `daily` |
| `client.api_keys()` | `create`, `list`, `get`, `update`, `revoke` |
| `client.quality()` | `scores`, `timeline` |
| `client.webhooks()` | `create`, `list`, `get`, `update`, `delete`, `test` |
| `client.billing()` | `subscription`, `checkout`, `portal`, `invoices`, `payment_method` |

## Error Handling

All methods return `Result<T, VynFiError>`. Match on error variants:

```rust
use vynfi::VynFiError;

match client.jobs().get("bad-id").await {
    Ok(job) => println!("Got job: {}", job.id),
    Err(VynFiError::NotFound(_)) => println!("Job not found"),
    Err(VynFiError::RateLimit(_)) => println!("Rate limited, retry later"),
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
