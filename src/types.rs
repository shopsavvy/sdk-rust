use serde::{Deserialize, Serialize};

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

/// API response metadata containing credit usage info
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiMeta {
    pub credits_used: i32,
    pub credits_remaining: i32,
    pub rate_limit_remaining: Option<i32>,
}

/// Standard API response wrapper
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
    pub meta: Option<ApiMeta>,
}

impl<T> ApiResponse<T> {
    /// Get credits used from meta object
    pub fn credits_used(&self) -> i32 {
        self.meta.as_ref().map(|m| m.credits_used).unwrap_or(0)
    }

    /// Get credits remaining from meta object
    pub fn credits_remaining(&self) -> i32 {
        self.meta.as_ref().map(|m| m.credits_remaining).unwrap_or(0)
    }
}

/// Product details information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProductDetails {
    pub title: String,
    pub shopsavvy: String,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub images: Option<Vec<String>>,
    pub barcode: Option<String>,
    pub amazon: Option<String>,
    pub model: Option<String>,
    pub mpn: Option<String>,
    pub color: Option<String>,
}

impl ProductDetails {
    /// Get product name (deprecated, use title)
    pub fn name(&self) -> &str {
        &self.title
    }

    /// Get product ID (deprecated, use shopsavvy)
    pub fn product_id(&self) -> &str {
        &self.shopsavvy
    }

    /// Get ASIN (deprecated, use amazon)
    pub fn asin(&self) -> Option<&str> {
        self.amazon.as_deref()
    }

    /// Get first image URL (deprecated, use images[0])
    pub fn image_url(&self) -> Option<&str> {
        self.images.as_ref().and_then(|imgs| imgs.first().map(|s| s.as_str()))
    }
}

/// Single price point in history
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PriceHistoryEntry {
    pub date: String,
    pub price: f64,
    pub availability: String,
}

/// Product offer from a retailer
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Offer {
    pub id: String,
    pub retailer: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub availability: Option<String>,
    pub condition: Option<String>,
    #[serde(rename = "URL")]
    pub url: Option<String>,
    pub seller: Option<String>,
    pub timestamp: Option<String>,
    pub history: Option<Vec<PriceHistoryEntry>>,
}

impl Offer {
    /// Get offer ID (deprecated, use id)
    pub fn offer_id(&self) -> &str {
        &self.id
    }

    /// Get offer URL (deprecated, use url)
    pub fn offer_url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Get last updated time (deprecated, use timestamp)
    pub fn last_updated(&self) -> Option<&str> {
        self.timestamp.as_deref()
    }
}

/// Product with nested offers (returned by offers endpoint)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProductWithOffers {
    pub title: String,
    pub shopsavvy: String,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub images: Option<Vec<String>>,
    pub barcode: Option<String>,
    pub amazon: Option<String>,
    pub model: Option<String>,
    pub mpn: Option<String>,
    pub color: Option<String>,
    pub offers: Vec<Offer>,
}

/// Offer with historical price data
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OfferWithHistory {
    pub id: String,
    pub retailer: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub availability: Option<String>,
    pub condition: Option<String>,
    #[serde(rename = "URL")]
    pub url: Option<String>,
    pub seller: Option<String>,
    pub timestamp: Option<String>,
    pub price_history: Vec<PriceHistoryEntry>,
}

/// Scheduled product monitoring information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScheduledProduct {
    pub product_id: String,
    pub identifier: String,
    pub frequency: String,
    pub retailer: Option<String>,
    pub created_at: String,
    pub last_refreshed: Option<String>,
}

/// Current billing period details
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsagePeriod {
    pub start_date: String,
    pub end_date: String,
    pub credits_used: i32,
    pub credits_limit: i32,
    pub credits_remaining: i32,
    pub requests_made: i32,
}

/// API usage and credit information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsageInfo {
    pub current_period: UsagePeriod,
    pub usage_percentage: f64,
}

impl UsageInfo {
    /// Get credits used (deprecated, use current_period.credits_used)
    pub fn credits_used(&self) -> i32 {
        self.current_period.credits_used
    }

    /// Get credits remaining (deprecated, use current_period.credits_remaining)
    pub fn credits_remaining(&self) -> i32 {
        self.current_period.credits_remaining
    }

    /// Get credits total (deprecated, use current_period.credits_limit)
    pub fn credits_total(&self) -> i32 {
        self.current_period.credits_limit
    }

    /// Get billing period start (deprecated, use current_period.start_date)
    pub fn billing_period_start(&self) -> &str {
        &self.current_period.start_date
    }

    /// Get billing period end (deprecated, use current_period.end_date)
    pub fn billing_period_end(&self) -> &str {
        &self.current_period.end_date
    }
}

/// Pagination info for search results
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PaginationInfo {
    pub total: i32,
    pub limit: i32,
    pub offset: i32,
    pub returned: i32,
}

/// Product search result with pagination
#[derive(Debug, Deserialize, Serialize)]
pub struct ProductSearchResult {
    pub success: bool,
    pub data: Vec<ProductDetails>,
    pub pagination: Option<PaginationInfo>,
    pub meta: Option<ApiMeta>,
}

impl ProductSearchResult {
    /// Get credits used from meta object
    pub fn credits_used(&self) -> i32 {
        self.meta.as_ref().map(|m| m.credits_used).unwrap_or(0)
    }

    /// Get credits remaining from meta object
    pub fn credits_remaining(&self) -> i32 {
        self.meta.as_ref().map(|m| m.credits_remaining).unwrap_or(0)
    }
}

/// Response from scheduling a product
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScheduleResponse {
    pub scheduled: bool,
    pub product_id: String,
}

/// Response from batch scheduling
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScheduleBatchResponse {
    pub identifier: String,
    pub scheduled: bool,
    pub product_id: String,
}

/// Response from removing a product from schedule
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RemoveResponse {
    pub removed: bool,
}

/// Response from batch removal
#[derive(Debug, Deserialize, Serialize, Clone)]
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
