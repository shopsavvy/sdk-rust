# ShopSavvy Data API - Rust SDK

[![Crates.io](https://img.shields.io/crates/v/shopsavvy-sdk.svg)](https://crates.io/crates/shopsavvy-sdk)
[![Rust Version](https://img.shields.io/badge/rustc-1.70+-blue.svg)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Documentation](https://img.shields.io/badge/docs-shopsavvy.com-blue)](https://shopsavvy.com/data/documentation)

Official Rust SDK for the [ShopSavvy Data API](https://shopsavvy.com/data). Access comprehensive product data, real-time pricing, and historical price trends across **thousands of retailers** and **millions of products**. Built for **high-performance**, **async/await**, and **zero-cost abstractions**.

## ⚡ 30-Second Quick Start

```toml
# Cargo.toml
[dependencies]
shopsavvy-sdk = "1.0.0"
tokio = { version = "1.0", features = ["full"] }
```

```rust
use shopsavvy_sdk::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("ss_live_your_api_key_here")?;
    
    let product = client.get_product_details("012345678901").await?;
    let offers = client.get_current_offers("012345678901").await?;
    let best_offer = offers.data.iter().min_by(|a, b| a.price.total_cmp(&b.price))?;
    
    println!("{} - Best price: ${:.2} at {}", product.data.name, best_offer.price, best_offer.retailer);
    Ok(())
}
```

## 📊 Feature Comparison

| Feature | Free Tier | Pro | Enterprise |
|---------|-----------|-----|-----------| 
| **API Calls/Month** | 1,000 | 100,000 | Unlimited |
| **Product Details** | ✅ | ✅ | ✅ |
| **Real-time Pricing** | ✅ | ✅ | ✅ |
| **Price History** | 30 days | 1 year | 5+ years |
| **Bulk Operations** | 10/batch | 100/batch | 1000/batch |
| **Retailer Coverage** | 50+ | 500+ | 1000+ |
| **Rate Limiting** | 60/hour | 1000/hour | Custom |
| **Support** | Community | Email | Phone + Dedicated |

## 🚀 Installation & Setup

### Requirements

- Rust 1.70+ (2021 edition)
- Tokio runtime for async operations
- TLS support (automatic with `reqwest`)

### Cargo Installation

```toml
[dependencies]
shopsavvy-sdk = "1.0.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }  # For custom serialization
```

### Optional Features

```toml
[dependencies]
shopsavvy-sdk = { version = "1.0.0", features = ["metrics", "tracing"] }
```

### Get Your API Key

1. **Sign up**: Visit [shopsavvy.com/data](https://shopsavvy.com/data)
2. **Choose plan**: Select based on your usage needs  
3. **Get API key**: Copy from your dashboard
4. **Test**: Run the 30-second example above

## 📖 Complete API Reference

### Client Configuration

```rust
use shopsavvy_sdk::{Client, Config, Result};
use std::time::Duration;
use std::env;

// Basic configuration
let client = Client::new("ss_live_your_api_key_here")?;

// Advanced configuration
let config = Config::builder()
    .api_key("ss_live_your_api_key_here")
    .base_url("https://api.shopsavvy.com/v1")
    .timeout(Duration::from_secs(60))
    .max_retries(3)
    .user_agent("MyApp/1.0.0")
    .build()?;

let client = Client::with_config(config)?;

// Environment variable configuration
let api_key = env::var("SHOPSAVVY_API_KEY")?;
let client = Client::new(&api_key)?;
```

### Product Lookup

#### Single Product
```rust
use shopsavvy_sdk::{Client, Result};

async fn lookup_product() -> Result<()> {
    let client = Client::new("ss_live_your_api_key_here")?;
    
    // Look up by barcode, ASIN, URL, model number, or ShopSavvy ID
    let product = client.get_product_details("012345678901").await?;
    let amazon_product = client.get_product_details("B08N5WRWNW").await?;
    let url_product = client.get_product_details("https://www.amazon.com/dp/B08N5WRWNW").await?;
    let model_product = client.get_product_details("MQ023LL/A").await?; // iPhone model number
    
    println!("📦 Product: {}", product.data.name);
    println!("🏷️ Brand: {}", product.data.brand.as_deref().unwrap_or("N/A"));
    println!("📂 Category: {}", product.data.category.as_deref().unwrap_or("N/A"));
    println!("🔢 Product ID: {}", product.data.id);
    
    if let Some(asin) = &product.data.asin {
        println!("📦 ASIN: {}", asin);
    }
    
    if let Some(model) = &product.data.model_number {
        println!("🔧 Model: {}", model);
    }
    
    Ok(())
}
```

#### Bulk Product Lookup with Concurrent Processing
```rust
use shopsavvy_sdk::{Client, Result};
use futures::future;
use std::collections::HashMap;

async fn lookup_multiple_products() -> Result<()> {
    let client = Client::new("ss_live_your_api_key_here")?;
    
    // Process up to 100 products at once (Pro plan)
    let identifiers = vec![
        "012345678901".to_string(),
        "B08N5WRWNW".to_string(), 
        "045496590048".to_string(),
        "https://www.bestbuy.com/site/product/123456".to_string(),
        "MQ023LL/A".to_string(),   // iPhone model number
        "SM-S911U".to_string(),    // Samsung model number
    ];
    
    // Method 1: Batch API (more efficient)
    let products = client.get_product_details_batch(&identifiers).await?;
    
    for (i, product) in products.data.iter().enumerate() {
        match product {
            Some(product) => {
                println!("✓ Found: {} by {}", 
                    product.name, 
                    product.brand.as_deref().unwrap_or("Unknown")
                );
            }
            None => {
                println!("❌ Failed to find product: {}", identifiers[i]);
            }
        }
    }
    
    // Method 2: Concurrent individual requests (for custom logic)
    let futures: Vec<_> = identifiers.iter()
        .map(|id| client.get_product_details(id))
        .collect();
    
    let results = future::join_all(futures).await;
    let mut product_map = HashMap::new();
    
    for (id, result) in identifiers.iter().zip(results.iter()) {
        match result {
            Ok(product) => {
                product_map.insert(id.clone(), product.data.clone());
                println!("✓ Concurrent lookup: {}", product.data.name);
            }
            Err(e) => {
                eprintln!("❌ Error for {}: {}", id, e);
            }
        }
    }
    
    Ok(())
}
```

### Real-Time Pricing with Advanced Analytics

#### High-Performance Price Analysis
```rust
use shopsavvy_sdk::{Client, Result};
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct PriceAnalysis {
    best_price: f64,
    worst_price: f64,
    average_price: f64,
    median_price: f64,
    price_variance: f64,
    total_offers: usize,
    in_stock_offers: usize,
    new_condition_offers: usize,
    retailer_rankings: BTreeMap<String, f64>,
}

async fn analyze_offers(client: &Client, identifier: &str) -> Result<PriceAnalysis> {
    let response = client.get_current_offers(identifier).await?;
    let offers = &response.data;
    
    println!("Found {} offers across retailers", offers.len());
    
    // Filter valid prices and sort
    let valid_offers: Vec<_> = offers.iter()
        .filter(|offer| offer.price > 0.0)
        .collect();
    
    if valid_offers.is_empty() {
        return Err("No valid offers found".into());
    }
    
    let mut prices: Vec<f64> = valid_offers.iter().map(|offer| offer.price).collect();
    prices.sort_by(|a, b| a.total_cmp(b));
    
    let best_price = prices[0];
    let worst_price = prices[prices.len() - 1];
    let average_price = prices.iter().sum::<f64>() / prices.len() as f64;
    
    // Calculate median
    let median_price = if prices.len() % 2 == 0 {
        (prices[prices.len() / 2 - 1] + prices[prices.len() / 2]) / 2.0
    } else {
        prices[prices.len() / 2]
    };
    
    // Calculate variance
    let variance = prices.iter()
        .map(|price| (price - average_price).powi(2))
        .sum::<f64>() / prices.len() as f64;
    
    println!("💰 Best price: ${:.2}", best_price);
    println!("💸 Highest price: ${:.2}", worst_price);
    println!("📊 Average price: ${:.2}", average_price);
    println!("📈 Median price: ${:.2}", median_price);
    println!("💡 Potential savings: ${:.2}", worst_price - best_price);
    
    // Additional analysis
    let in_stock_offers = offers.iter()
        .filter(|offer| offer.availability.as_deref() == Some("in_stock"))
        .count();
    
    let new_condition_offers = offers.iter()
        .filter(|offer| offer.condition.as_deref() == Some("new"))
        .count();
    
    println!("✅ In-stock offers: {}", in_stock_offers);
    println!("🆕 New condition: {}", new_condition_offers);
    
    // Retailer rankings
    let mut retailer_rankings = BTreeMap::new();
    for offer in valid_offers {
        retailer_rankings.insert(offer.retailer.clone(), offer.price);
    }
    
    Ok(PriceAnalysis {
        best_price,
        worst_price,
        average_price,
        median_price,
        price_variance: variance,
        total_offers: offers.len(),
        in_stock_offers,
        new_condition_offers,
        retailer_rankings,
    })
}
```

#### Retailer-Specific Price Comparison
```rust
use shopsavvy_sdk::{Client, Result};
use std::collections::HashMap;
use futures::future;

async fn compare_retailer_prices(client: &Client, identifier: &str) -> Result<()> {
    let retailers = vec!["amazon", "walmart", "target", "bestbuy"];
    
    // Concurrent retailer queries
    let futures: Vec<_> = retailers.iter()
        .map(|retailer| client.get_current_offers_with_retailer(identifier, retailer))
        .collect();
    
    let results = future::join_all(futures).await;
    let mut retailer_prices = HashMap::new();
    
    for (retailer, result) in retailers.iter().zip(results.iter()) {
        match result {
            Ok(offers) if !offers.data.is_empty() => {
                let best_price = offers.data.iter()
                    .filter(|offer| offer.price > 0.0)
                    .map(|offer| offer.price)
                    .fold(f64::INFINITY, f64::min);
                
                if best_price != f64::INFINITY {
                    retailer_prices.insert(retailer.to_string(), best_price);
                }
            }
            Ok(_) => {
                println!("No offers found for {}", retailer);
            }
            Err(e) => {
                eprintln!("Error fetching {} prices: {}", retailer, e);
            }
        }
    }
    
    // Sort by price
    let mut sorted_prices: Vec<_> = retailer_prices.iter().collect();
    sorted_prices.sort_by(|a, b| a.1.total_cmp(b.1));
    
    println!("Retailer price comparison:");
    for (retailer, price) in sorted_prices {
        println!("  {}: ${:.2}", 
            retailer.chars().next().unwrap().to_uppercase().collect::<String>() 
                + &retailer[1..], 
            price
        );
    }
    
    Ok(())
}
```

## 🚀 Production Deployment

### High-Performance Web Service with Axum

```rust
// Cargo.toml
[dependencies]
shopsavvy-sdk = "1.0.0"
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tracing = "0.1"
tracing-subscriber = "0.3"

// src/main.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use shopsavvy_sdk::{Client, Result as SdkResult};
use std::{collections::HashMap, sync::Arc};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    client: Arc<Client>,
}

#[derive(Deserialize)]
struct OffersQuery {
    retailer: Option<String>,
    sort_by: Option<String>,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    credits_remaining: Option<i32>,
}

impl<T> ApiResponse<T> {
    fn ok(data: T, credits: Option<i32>) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            credits_remaining: credits,
        }
    }
    
    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            credits_remaining: None,
        }
    }
}

async fn get_product_offers(
    State(state): State<AppState>,
    Path(identifier): Path<String>,
    Query(params): Query<OffersQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    match state.client.get_current_offers(&identifier).await {
        Ok(response) => {
            let mut offers = response.data;
            
            // Filter by retailer if specified
            if let Some(retailer) = &params.retailer {
                offers.retain(|offer| offer.retailer.to_lowercase() == retailer.to_lowercase());
            }
            
            // Sort offers
            match params.sort_by.as_deref() {
                Some("price") => offers.sort_by(|a, b| a.price.total_cmp(&b.price)),
                Some("retailer") => offers.sort_by(|a, b| a.retailer.cmp(&b.retailer)),
                _ => {} // Default order
            }
            
            let analysis = serde_json::json!({
                "offers": offers,
                "summary": {
                    "total_offers": offers.len(),
                    "best_price": offers.iter().map(|o| o.price).fold(f64::INFINITY, f64::min),
                    "average_price": offers.iter().map(|o| o.price).sum::<f64>() / offers.len() as f64,
                    "in_stock_count": offers.iter().filter(|o| o.availability.as_deref() == Some("in_stock")).count(),
                }
            });
            
            Ok(Json(ApiResponse::ok(analysis, response.credits_remaining)))
        }
        Err(e) => {
            eprintln!("Error fetching offers: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

async fn compare_products(
    State(state): State<AppState>,
    Json(identifiers): Json<Vec<String>>,
) -> Result<Json<ApiResponse<HashMap<String, serde_json::Value>>>, StatusCode> {
    if identifiers.len() > 50 {
        return Ok(Json(ApiResponse::error("Maximum 50 products allowed".to_string())));
    }
    
    let futures: Vec<_> = identifiers.iter()
        .map(|id| async {
            let product_result = state.client.get_product_details(id).await;
            let offers_result = state.client.get_current_offers(id).await;
            
            match (product_result, offers_result) {
                (Ok(product), Ok(offers)) => {
                    let best_offer = offers.data.iter()
                        .min_by(|a, b| a.price.total_cmp(&b.price));
                    
                    Some((id.clone(), serde_json::json!({
                        "product": product.data,
                        "best_offer": best_offer,
                        "total_offers": offers.data.len(),
                    })))
                }
                _ => None,
            }
        })
        .collect();
    
    let results = futures::future::join_all(futures).await;
    let comparison: HashMap<String, serde_json::Value> = results
        .into_iter()
        .filter_map(|result| result)
        .collect();
    
    Ok(Json(ApiResponse::ok(comparison, None)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::init();
    
    let client = Client::new(&std::env::var("SHOPSAVVY_API_KEY")?)?;
    let state = AppState {
        client: Arc::new(client),
    };
    
    let app = Router::new()
        .route("/products/:identifier/offers", get(get_product_offers))
        .route("/products/compare", post(compare_products))
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Server running on http://localhost:3000");
    
    axum::serve(listener, app).await?;
    Ok(())
}
```

### Background Price Monitoring Service

```rust
use shopsavvy_sdk::{Client, Result};
use tokio::{time::{sleep, Duration}, task, sync::mpsc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PriceAlert {
    product_id: String,
    product_name: String,
    target_price: f64,
    current_price: f64,
    retailer: String,
    user_id: String,
}

#[derive(Debug, Clone)]
struct MonitoredProduct {
    identifier: String,
    target_price: f64,
    user_id: String,
}

struct PriceMonitoringService {
    client: Client,
    products: Vec<MonitoredProduct>,
    alert_sender: mpsc::UnboundedSender<PriceAlert>,
}

impl PriceMonitoringService {
    fn new(client: Client) -> (Self, mpsc::UnboundedReceiver<PriceAlert>) {
        let (alert_sender, alert_receiver) = mpsc::unbounded_channel();
        
        (
            Self {
                client,
                products: Vec::new(),
                alert_sender,
            },
            alert_receiver,
        )
    }
    
    fn add_product(&mut self, identifier: String, target_price: f64, user_id: String) {
        self.products.push(MonitoredProduct {
            identifier,
            target_price,
            user_id,
        });
    }
    
    async fn start_monitoring(&self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Check hourly
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_all_prices().await {
                eprintln!("Error during price check: {}", e);
            }
        }
    }
    
    async fn check_all_prices(&self) -> Result<()> {
        // Process products in batches to respect rate limits
        const BATCH_SIZE: usize = 10;
        
        for batch in self.products.chunks(BATCH_SIZE) {
            let futures: Vec<_> = batch.iter()
                .map(|product| self.check_product_price(product))
                .collect();
            
            let results = futures::future::join_all(futures).await;
            
            for result in results {
                if let Err(e) = result {
                    eprintln!("Price check error: {}", e);
                }
            }
            
            // Rate limiting between batches
            sleep(Duration::from_secs(1)).await;
        }
        
        Ok(())
    }
    
    async fn check_product_price(&self, product: &MonitoredProduct) -> Result<()> {
        let offers = self.client.get_current_offers(&product.identifier).await?;
        
        if let Some(best_offer) = offers.data.iter()
            .filter(|offer| offer.price > 0.0)
            .min_by(|a, b| a.price.total_cmp(&b.price))
        {
            if best_offer.price <= product.target_price {
                // Get product details for the alert
                if let Ok(product_details) = self.client.get_product_details(&product.identifier).await {
                    let alert = PriceAlert {
                        product_id: product.identifier.clone(),
                        product_name: product_details.data.name,
                        target_price: product.target_price,
                        current_price: best_offer.price,
                        retailer: best_offer.retailer.clone(),
                        user_id: product.user_id.clone(),
                    };
                    
                    if let Err(e) = self.alert_sender.send(alert) {
                        eprintln!("Failed to send alert: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }
}

async fn handle_alerts(mut alert_receiver: mpsc::UnboundedReceiver<PriceAlert>) {
    while let Some(alert) = alert_receiver.recv().await {
        println!("🚨 PRICE ALERT: {} is now ${:.2} at {} (target: ${:.2})", 
            alert.product_name, 
            alert.current_price, 
            alert.retailer, 
            alert.target_price
        );
        
        // Here you would:
        // - Send email notification
        // - Send push notification
        // - Update database
        // - Send webhook to external service
        
        // Example: Send to webhook
        if let Err(e) = send_webhook_alert(&alert).await {
            eprintln!("Failed to send webhook: {}", e);
        }
    }
}

async fn send_webhook_alert(alert: &PriceAlert) -> Result<()> {
    let webhook_url = std::env::var("WEBHOOK_URL")?;
    let client = reqwest::Client::new();
    
    let payload = serde_json::json!({
        "type": "price_alert",
        "data": alert,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    
    client
        .post(&webhook_url)
        .json(&payload)
        .send()
        .await?;
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new(&std::env::var("SHOPSAVVY_API_KEY")?)?;
    let (mut monitor, alert_receiver) = PriceMonitoringService::new(client);
    
    // Add products to monitor
    monitor.add_product("012345678901".to_string(), 99.99, "user123".to_string());
    monitor.add_product("B08N5WRWNW".to_string(), 199.99, "user456".to_string());
    
    // Start alert handler
    let alert_handler = task::spawn(handle_alerts(alert_receiver));
    
    // Start monitoring
    let monitoring = task::spawn(async move {
        if let Err(e) = monitor.start_monitoring().await {
            eprintln!("Monitoring error: {}", e);
        }
    });
    
    println!("🔍 Price monitoring service started");
    
    // Wait for either task to complete (they shouldn't in normal operation)
    tokio::select! {
        _ = alert_handler => println!("Alert handler stopped"),
        _ = monitoring => println!("Monitoring stopped"),
    }
    
    Ok(())
}
```

### WebAssembly Integration

```rust
// Cargo.toml
[lib]
crate-type = ["cdylib"]

[dependencies]
shopsavvy-sdk = "1.0.0"
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
serde-wasm-bindgen = "0.6"

// src/lib.rs
use wasm_bindgen::prelude::*;
use shopsavvy_sdk::{Client, Result};
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Serialize, Deserialize)]
pub struct ProductComparison {
    pub product_name: String,
    pub best_price: f64,
    pub best_retailer: String,
    pub total_offers: usize,
    pub savings_opportunity: f64,
}

#[wasm_bindgen]
pub struct ShopSavvyWasm {
    client: Client,
}

#[wasm_bindgen]
impl ShopSavvyWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(api_key: &str) -> Result<ShopSavvyWasm, JsValue> {
        console_error_panic_hook::set_once();
        
        let client = Client::new(api_key)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(ShopSavvyWasm { client })
    }
    
    #[wasm_bindgen]
    pub async fn compare_product_prices(
        &self, 
        identifier: &str
    ) -> Result<JsValue, JsValue> {
        let offers = self.client.get_current_offers(identifier)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let product = self.client.get_product_details(identifier)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        if offers.data.is_empty() {
            return Err(JsValue::from_str("No offers found"));
        }
        
        let best_offer = offers.data.iter()
            .min_by(|a, b| a.price.total_cmp(&b.price))
            .unwrap();
        
        let worst_offer = offers.data.iter()
            .max_by(|a, b| a.price.total_cmp(&b.price))
            .unwrap();
        
        let comparison = ProductComparison {
            product_name: product.data.name,
            best_price: best_offer.price,
            best_retailer: best_offer.retailer.clone(),
            total_offers: offers.data.len(),
            savings_opportunity: worst_offer.price - best_offer.price,
        };
        
        Ok(serde_wasm_bindgen::to_value(&comparison)?)
    }
}

// JavaScript usage:
// import init, { ShopSavvyWasm } from './pkg/my_wasm_app.js';
// 
// async function run() {
//   await init();
//   const client = new ShopSavvyWasm('your_api_key');
//   const comparison = await client.compare_product_prices('012345678901');
//   console.log(comparison);
// }
```

## Error Handling

The SDK provides comprehensive error handling with Result types and custom error variants:

```rust
use shopsavvy_sdk::{Client, Error, Result};

async fn handle_product_lookup(identifier: &str) -> Result<()> {
    let client = Client::new("ss_live_your_api_key_here")?;
    
    match client.get_product_details(identifier).await {
        Ok(product) => {
            println!("✅ Found product: {}", product.data.name);
        }
        Err(Error::Authentication { message, .. }) => {
            eprintln!("🔐 Authentication failed: {}", message);
            // Redirect to login or refresh token
        }
        Err(Error::NotFound { message, .. }) => {
            eprintln!("❌ Product not found: {}", message);
            // Show "not found" UI
        }
        Err(Error::Validation { message, .. }) => {
            eprintln!("⚠️ Invalid parameters: {}", message);
            // Show validation error to user
        }
        Err(Error::RateLimit { message, retry_after, .. }) => {
            eprintln!("🚦 Rate limit exceeded: {}", message);
            if let Some(delay) = retry_after {
                eprintln!("Retry after: {} seconds", delay);
            }
            // Implement exponential backoff
        }
        Err(Error::Network { message, .. }) => {
            eprintln!("🌐 Network error: {}", message);
            // Show offline mode or retry option
        }
        Err(Error::Timeout { .. }) => {
            eprintln!("⏰ Request timed out");
            // Retry with exponential backoff
        }
        Err(e) => {
            eprintln!("💥 Unexpected error: {}", e);
            // Log to crash reporting service
        }
    }
    
    Ok(())
}
```

### Retry Logic with Exponential Backoff

```rust
use shopsavvy_sdk::{Client, Error, Result};
use tokio::time::{sleep, Duration};
use std::cmp;

async fn retry_with_backoff<F, T>(
    mut operation: F,
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
) -> Result<T>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
{
    let mut attempt = 1;
    let mut delay = initial_delay;
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(Error::RateLimit { retry_after, .. }) => {
                if attempt >= max_attempts {
                    return Err(Error::RateLimit { 
                        message: "Max retry attempts exceeded".to_string(),
                        retry_after,
                    });
                }
                
                // Use server-specified retry delay if available
                let retry_delay = retry_after
                    .map(Duration::from_secs)
                    .unwrap_or(delay);
                
                eprintln!("Rate limited, retrying in {:?}...", retry_delay);
                sleep(retry_delay).await;
            }
            Err(Error::Network { .. }) | Err(Error::Timeout { .. }) => {
                if attempt >= max_attempts {
                    return Err(Error::Network { 
                        message: "Max retry attempts exceeded".to_string() 
                    });
                }
                
                eprintln!("Network error, retrying in {:?}...", delay);
                sleep(delay).await;
            }
            Err(e) => return Err(e), // Don't retry other errors
        }
        
        attempt += 1;
        delay = cmp::min(delay * 2, max_delay); // Exponential backoff with cap
    }
}

// Usage
async fn robust_product_lookup(client: &Client, identifier: &str) -> Result<Product> {
    retry_with_backoff(
        || Box::pin(client.get_product_details(identifier)),
        3,
        Duration::from_secs(1),
        Duration::from_secs(30),
    ).await
}
```

## 🛠️ Development & Testing

### Local Development Setup

```bash
# Clone the repository
git clone https://github.com/shopsavvy/sdk-rust.git
cd sdk-rust

# Build the project
cargo build

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Check formatting
cargo fmt --check

# Run Clippy lints
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open
```

### Testing Your Integration

```rust
use shopsavvy_sdk::{Client, Result};

#[tokio::test]
async fn test_sdk_integration() -> Result<()> {
    // Use test API key (starts with ss_test_)
    let client = Client::new("ss_test_your_test_key_here")?;
    
    // Test product lookup
    let product = client.get_product_details("012345678901").await?;
    println!("✅ Product lookup: {}", product.data.name);
    assert!(!product.data.name.is_empty());
    
    // Test current offers
    let offers = client.get_current_offers("012345678901").await?;
    println!("✅ Current offers: {} found", offers.data.len());
    assert!(!offers.data.is_empty());
    
    // Test usage info
    let usage = client.get_usage().await?;
    println!("✅ API usage: {} credits remaining", 
        usage.data.credits_remaining.unwrap_or(0));
    
    println!("\n🎉 All tests passed! SDK is working correctly.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    test_sdk_integration().await
}
```

## Data Models

All data structures use Rust's type system for safety and performance:

### ProductDetails
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDetails {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub upc: Option<String>,
    pub asin: Option<String>,
    pub model_number: Option<String>,
    pub images: Vec<String>,
    pub specifications: HashMap<String, String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl ProductDetails {
    /// Check if the product has images
    pub fn has_images(&self) -> bool {
        !self.images.is_empty()
    }
    
    /// Get the display name (brand + name)
    pub fn display_name(&self) -> String {
        match &self.brand {
            Some(brand) => format!("{} {}", brand, self.name),
            None => self.name.clone(),
        }
    }
    
    /// Get the main product image
    pub fn main_image(&self) -> Option<&String> {
        self.images.first()
    }
}
```

### Offer
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Offer {
    pub retailer: String,
    pub price: f64,
    pub currency: Option<String>,
    pub availability: Option<String>,
    pub condition: Option<String>,
    pub shipping_cost: Option<f64>,
    pub url: Option<String>,
    pub last_updated: Option<String>,
}

impl Offer {
    /// Check if the offer is in stock
    pub fn is_in_stock(&self) -> bool {
        self.availability.as_deref() == Some("in_stock")
    }
    
    /// Check if the condition is new
    pub fn is_new_condition(&self) -> bool {
        self.condition.as_deref() == Some("new")
    }
    
    /// Calculate total cost including shipping
    pub fn total_cost(&self) -> f64 {
        self.price + self.shipping_cost.unwrap_or(0.0)
    }
    
    /// Format price with currency
    pub fn formatted_price(&self) -> String {
        let currency = self.currency.as_deref().unwrap_or("USD");
        match currency {
            "USD" => format!("${:.2}", self.price),
            "EUR" => format!("€{:.2}", self.price),
            "GBP" => format!("£{:.2}", self.price),
            _ => format!("{} {:.2}", currency, self.price),
        }
    }
}
```

### ApiResponse
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
    pub credits_used: Option<i32>,
    pub credits_remaining: Option<i32>,
}

impl<T> ApiResponse<T> {
    /// Check if the response was successful
    pub fn is_success(&self) -> bool {
        self.success
    }
    
    /// Get credits remaining or default to 0
    pub fn credits_remaining(&self) -> i32 {
        self.credits_remaining.unwrap_or(0)
    }
}
```

## 📚 Additional Resources

- **[ShopSavvy Data API Documentation](https://shopsavvy.com/data/documentation)** - Complete API reference
- **[API Dashboard](https://shopsavvy.com/data/dashboard)** - Manage your API keys and usage
- **[GitHub Repository](https://github.com/shopsavvy/sdk-rust)** - Source code and issues
- **[Crates.io](https://crates.io/crates/shopsavvy-sdk)** - Package releases and stats
- **[Rust Documentation](https://doc.rust-lang.org/)** - Rust language reference
- **[Tokio Documentation](https://tokio.rs/)** - Async runtime documentation
- **[Support](mailto:business@shopsavvy.com)** - Get help from our team

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on:

- Reporting bugs and feature requests
- Setting up development environment  
- Submitting pull requests
- Code standards and testing
- Rust best practices and performance optimization

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🏢 About ShopSavvy

**ShopSavvy** is the world's first mobile shopping app, helping consumers find the best deals since 2008. With over **40 million downloads** and millions of active users, ShopSavvy has saved consumers billions of dollars.

### Our Data API Powers:
- 🛒 **E-commerce platforms** with competitive intelligence  
- 📊 **Market research** with real-time pricing data
- 🏪 **Retailers** with inventory and pricing optimization
- 📱 **Mobile apps** with product lookup and price comparison
- 🤖 **Business intelligence** with automated price monitoring

### Why Choose ShopSavvy Data API?
- ✅ **Trusted by millions** - Proven at scale since 2008
- ✅ **Comprehensive coverage** - 1000+ retailers, millions of products  
- ✅ **Real-time accuracy** - Fresh data updated continuously
- ✅ **Developer-friendly** - Easy integration, great documentation
- ✅ **Reliable infrastructure** - 99.9% uptime, enterprise-grade
- ✅ **Flexible pricing** - Plans for every use case and budget

### Perfect for Rust Development:
- 🚀 **Zero-cost abstractions** - Maximum performance with minimal overhead
- ⚡ **Async/await first** - Built on Tokio for high-concurrency applications
- 🛡️ **Memory safety** - Leverage Rust's ownership system for bulletproof code
- 🔧 **Rich type system** - Compile-time guarantees and excellent IDE support
- 🌐 **WebAssembly ready** - Compile to WASM for browser and edge deployment
- 📊 **High performance** - Ideal for data-intensive and real-time applications

---

**Ready to get started?** [Sign up for your API key](https://shopsavvy.com/data) • **Need help?** [Contact us](mailto:business@shopsavvy.com)