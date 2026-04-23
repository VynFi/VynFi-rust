//! VynFi Rust SDK — synthetic financial data generation.
//!
//! # Example
//! ```no_run
//! use vynfi::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), vynfi::VynFiError> {
//!     let client = Client::builder("vf_live_...").build()?;
//!     let sectors = client.catalog().list_sectors().await?;
//!     Ok(())
//! }
//! ```

mod archive;
mod client;
mod error;
mod resources;
mod types;

#[cfg(feature = "blocking")]
pub mod blocking;

pub use archive::JobArchive;
pub use client::{Client, ClientBuilder};
pub use error::{ErrorBody, VynFiError};
pub use resources::{
    Adversarial, Ai, ApiKeys, Billing, Catalog, Configs, Credits, Fingerprint, Jobs,
    ListConfigsParams, ListJobsParams, ListNotificationsParams, NdjsonStreamParams, Notifications,
    Optimizer, Quality, Scenarios, Sessions, TemplatePacks, Usage, Webhooks,
};
pub use types::*;
