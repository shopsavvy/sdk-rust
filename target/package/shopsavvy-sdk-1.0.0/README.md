# ShopSavvy Rust SDK

Official Rust SDK for the ShopSavvy Data API - Access product data, pricing information, and price history across thousands of retailers and millions of products.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
shopsavvy-data-api = "1.0.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use shopsavvy_data_api::{Client, OutputFormat, MonitoringFrequency};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new client
    let client = Client::new("ss_live_your_api_key_here")?;
    
    // Look up a product by barcode
    let product = client.get_product_details("012345678901", None).await?;
    println!("Product: {}", product.data.name);
    
    if let Some(brand) = &product.data.brand {
        println!("Brand: {}", brand);
    }
    
    // Get current offers
    let offers = client.get_current_offers("012345678901", None, None).await?;
    println!("Found {} offers:", offers.data.len());
    
    for offer in offers.data {
        println!("  {}: ${:.2} ({})", offer.retailer, offer.price, offer.availability);
    }
    
    Ok(())
}
```

## Configuration

You can customize the client behavior:

```rust
use shopsavvy_data_api::{Client, Config};
use std::time::Duration;

let config = Config::new("ss_live_your_api_key_here")
    .with_timeout(Duration::from_secs(60))
    .with_base_url("https://api.shopsavvy.com/v1");

let client = Client::with_config(config)?;
```

## API Methods

### Product Details

```rust
// Single product
let product = client.get_product_details("012345678901", None).await?;

// Multiple products
let products = client.get_product_details_batch(
    &["012345678901", "B08N5WRWNW"], 
    None
).await?;

// CSV format
let product_csv = client.get_product_details(
    "012345678901", 
    Some(OutputFormat::Csv)
).await?;
```

### Current Offers

```rust
// All retailers
let offers = client.get_current_offers("012345678901", None, None).await?;

// Specific retailer
let amazon_offers = client.get_current_offers(
    "012345678901", 
    Some("amazon"), 
    None
).await?;

// Multiple products
let offers_batch = client.get_current_offers_batch(
    &["012345678901", "B08N5WRWNW"], 
    None, 
    None
).await?;
```

### Price History

```rust
let history = client.get_price_history(
    "012345678901",
    "2024-01-01",
    "2024-01-31",
    None,
    None
).await?;

// With specific retailer
let amazon_history = client.get_price_history(
    "012345678901",
    "2024-01-01", 
    "2024-01-31",
    Some("amazon"),
    None
).await?;
```

### Product Monitoring

```rust
use shopsavvy_data_api::MonitoringFrequency;

// Schedule monitoring
let result = client.schedule_product_monitoring(
    "012345678901",
    MonitoringFrequency::Daily,
    None
).await?;

// Schedule multiple products
let results = client.schedule_product_monitoring_batch(
    &["012345678901", "B08N5WRWNW"],
    MonitoringFrequency::Daily,
    None
).await?;

// Get scheduled products
let scheduled = client.get_scheduled_products().await?;

// Remove from schedule
let removed = client.remove_product_from_schedule("012345678901").await?;

// Remove multiple from schedule
let removed_batch = client.remove_products_from_schedule(
    &["012345678901", "B08N5WRWNW"]
).await?;
```

### Usage Information

```rust
let usage = client.get_usage().await?;
println!("Credits remaining: {}", usage.data.credits_remaining);
```

## Error Handling

The SDK provides specific error types:

```rust
use shopsavvy_data_api::Error;

match client.get_product_details("invalid-id", None).await {
    Ok(product) => println!("Product: {}", product.data.name),
    Err(Error::Authentication { message, .. }) => {
        println!("Invalid API key: {}", message);
    },
    Err(Error::NotFound { message, .. }) => {
        println!("Product not found: {}", message);
    },
    Err(Error::Validation { message, .. }) => {
        println!("Invalid request: {}", message);
    },
    Err(Error::RateLimit { message, .. }) => {
        println!("Rate limit exceeded: {}", message);
    },
    Err(e) => println!("API error: {}", e),
}
```

## Types

### Output Formats

```rust
use shopsavvy_data_api::OutputFormat;

// Available formats
OutputFormat::Json
OutputFormat::Csv
```

### Monitoring Frequencies

```rust
use shopsavvy_data_api::MonitoringFrequency;

// Available frequencies
MonitoringFrequency::Hourly
MonitoringFrequency::Daily
MonitoringFrequency::Weekly
```

## Response Structure

All API responses follow the same structure:

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
    pub credits_used: Option<i32>,
    pub credits_remaining: Option<i32>,
}
```

## Requirements

- Rust 1.70 or higher
- Valid ShopSavvy API key (get one at [shopsavvy.com/data](https://shopsavvy.com/data))

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Links

- [ShopSavvy Data API](https://shopsavvy.com/data)
- [API Documentation](https://shopsavvy.com/data/documentation)
- [Get API Key](https://shopsavvy.com/data)
- [Report Issues](https://github.com/shopsavvy/rust-sdk/issues)