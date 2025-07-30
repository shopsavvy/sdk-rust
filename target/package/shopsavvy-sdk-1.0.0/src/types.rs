use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the ShopSavvy API client
#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub base_url: String,
    pub timeout: std::time::Duration,
}

impl Config {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.shopsavvy.com/v1".to_string(),
            timeout: std::time::Duration::from_secs(30),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Standard API response wrapper
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
    pub credits_used: Option<i32>,
    pub credits_remaining: Option<i32>,
}

/// Product details information
#[derive(Debug, Deserialize, Serialize)]
pub struct ProductDetails {
    pub product_id: String,
    pub name: String,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub image_url: Option<String>,
    pub barcode: Option<String>,
    pub asin: Option<String>,
    pub model: Option<String>,
    pub mpn: Option<String>,
    pub description: Option<String>,
    pub identifiers: Option<HashMap<String, String>>,
}

/// Product offer from a retailer
#[derive(Debug, Deserialize, Serialize)]
pub struct Offer {
    pub offer_id: String,
    pub retailer: String,
    pub price: f64,
    pub currency: String,
    pub availability: String,
    pub condition: String,
    pub url: String,
    pub shipping: Option<f64>,
    pub last_updated: String,
}

/// Single price point in history
#[derive(Debug, Deserialize, Serialize)]
pub struct PriceHistoryEntry {
    pub date: String,
    pub price: f64,
    pub availability: String,
}

/// Offer with historical price data
#[derive(Debug, Deserialize, Serialize)]
pub struct OfferWithHistory {
    #[serde(flatten)]
    pub offer: Offer,
    pub price_history: Vec<PriceHistoryEntry>,
}

/// Scheduled product monitoring information
#[derive(Debug, Deserialize, Serialize)]
pub struct ScheduledProduct {
    pub product_id: String,
    pub identifier: String,
    pub frequency: String,
    pub retailer: Option<String>,
    pub created_at: String,
    pub last_refreshed: Option<String>,
}

/// API usage and credit information
#[derive(Debug, Deserialize, Serialize)]
pub struct UsageInfo {
    pub credits_used: i32,
    pub credits_remaining: i32,
    pub credits_total: i32,
    pub billing_period_start: String,
    pub billing_period_end: String,
    pub plan_name: String,
}

/// Response from scheduling a product
#[derive(Debug, Deserialize, Serialize)]
pub struct ScheduleResponse {
    pub scheduled: bool,
    pub product_id: String,
}

/// Response from batch scheduling
#[derive(Debug, Deserialize, Serialize)]
pub struct ScheduleBatchResponse {
    pub identifier: String,
    pub scheduled: bool,
    pub product_id: String,
}

/// Response from removing a product from schedule
#[derive(Debug, Deserialize, Serialize)]
pub struct RemoveResponse {
    pub removed: bool,
}

/// Response from batch removal
#[derive(Debug, Deserialize, Serialize)]
pub struct RemoveBatchResponse {
    pub identifier: String,
    pub removed: bool,
}

/// Available output formats
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Json,
    Csv,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Csv => write!(f, "csv"),
        }
    }
}

/// Available monitoring frequencies
#[derive(Debug, Clone)]
pub enum MonitoringFrequency {
    Hourly,
    Daily,
    Weekly,
}

impl std::fmt::Display for MonitoringFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitoringFrequency::Hourly => write!(f, "hourly"),
            MonitoringFrequency::Daily => write!(f, "daily"),
            MonitoringFrequency::Weekly => write!(f, "weekly"),
        }
    }
}