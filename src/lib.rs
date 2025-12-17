//! # ShopSavvy Rust SDK
//!
//! Official Rust SDK for the ShopSavvy Data API - Access product data, pricing information,
//! and price history across thousands of retailers and millions of products.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use shopsavvy_sdk::{Client, Config};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("ss_live_your_api_key_here")?;
//!
//!     let product = client.get_product_details("012345678901", None).await?;
//!     println!("Product: {}", product.data[0].title);
//!
//!     let offers = client.get_current_offers("012345678901", None, None).await?;
//!     println!("Found {} offers", offers.data[0].offers.len());
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod types;

pub use client::Client;
pub use error::{Error, Result};
pub use types::*;