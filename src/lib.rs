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

mod client;
mod error;
mod resources;
mod types;

#[cfg(feature = "blocking")]
pub mod blocking;

pub use client::{Client, ClientBuilder};
pub use error::{ErrorBody, VynFiError};
pub use resources::{
    ApiKeys, Billing, Catalog, Configs, Credits, Jobs, ListConfigsParams, ListJobsParams,
    ListNotificationsParams, Notifications, Quality, Scenarios, Sessions, Usage, Webhooks,
};
pub use types::*;
