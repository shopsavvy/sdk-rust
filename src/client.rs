use crate::{
    error::{Error, Result},
    types::*,
};
use regex::Regex;
use reqwest::{header::HeaderMap, Client as HttpClient};
use serde_json::Value;

/// SDK version
pub const VERSION: &str = "1.0.1";

/// ShopSavvy Data API client
#[derive(Debug, Clone)]
pub struct Client {
    config: Config,
    client: HttpClient,
}

impl Client {
    /// Create a new ShopSavvy Data API client
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your ShopSavvy API key
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use shopsavvy_sdk::Client;
    ///
    /// let client = Client::new("ss_live_your_api_key_here").unwrap();
    /// ```
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        let config = Config::new(api_key);
        Self::with_config(config)
    }

    /// Create a new client with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use shopsavvy_sdk::{Client, Config};
    /// use std::time::Duration;
    ///
    /// let config = Config::new("ss_live_your_api_key_here")
    ///     .with_timeout(Duration::from_secs(60));
    /// let client = Client::with_config(config).unwrap();
    /// ```
    pub fn with_config(config: Config) -> Result<Self> {
        // Validate API key
        if config.api_key.is_empty() {
            return Err(Error::MissingApiKey);
        }

        let api_key_regex = Regex::new(r"^ss_(live|test)_[a-zA-Z0-9]+$").unwrap();
        if !api_key_regex.is_match(&config.api_key) {
            return Err(Error::InvalidApiKey);
        }

        // Create HTTP headers
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", config.api_key).parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("User-Agent", format!("ShopSavvy-Rust-SDK/{}", VERSION).parse().unwrap());

        // Create HTTP client
        let client = HttpClient::builder()
            .timeout(config.timeout)
            .default_headers(headers)
            .build()?;

        Ok(Self { config, client })
    }

    /// Make an HTTP request and handle the response
    async fn request<T>(&self, method: reqwest::Method, endpoint: &str, params: Option<&[(&str, &str)]>, body: Option<&Value>) -> Result<ApiResponse<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let url = format!("{}{}", self.config.base_url, endpoint);

        let mut request = self.client.request(method, &url);

        if let Some(params) = params {
            request = request.query(params);
        }

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;
        let status_code = response.status().as_u16();

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            let error_message = if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                error_json["error"].as_str().unwrap_or(&error_text).to_string()
            } else {
                error_text
            };
            return Err(Error::from_status_code(status_code, error_message));
        }

        let response_text = response.text().await?;
        let api_response: ApiResponse<T> = serde_json::from_str(&response_text)?;

        Ok(api_response)
    }

    /// Make a request and return raw result (for ProductSearchResult)
    async fn request_raw<T>(&self, method: reqwest::Method, endpoint: &str, params: Option<&[(&str, &str)]>) -> Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let url = format!("{}{}", self.config.base_url, endpoint);

        let mut request = self.client.request(method, &url);

        if let Some(params) = params {
            request = request.query(params);
        }

        let response = request.send().await?;
        let status_code = response.status().as_u16();

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            let error_message = if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                error_json["error"].as_str().unwrap_or(&error_text).to_string()
            } else {
                error_text
            };
            return Err(Error::from_status_code(status_code, error_message));
        }

        let response_text = response.text().await?;
        let result: T = serde_json::from_str(&response_text)?;

        Ok(result)
    }

    /// Search for products by keyword
    ///
    /// # Arguments
    ///
    /// * `query` - Search query or keyword
    /// * `limit` - Optional maximum number of results
    /// * `offset` - Optional pagination offset
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let results = client.search_products("iphone 15 pro", Some(10), None).await?;
    /// for product in results.data {
    ///     println!("Product: {}", product.title);
    /// }
    /// ```
    pub async fn search_products(&self, query: &str, limit: Option<i32>, offset: Option<i32>) -> Result<ProductSearchResult> {
        let mut params = vec![("q", query)];

        let limit_str: String;
        if let Some(l) = limit {
            limit_str = l.to_string();
            params.push(("limit", &limit_str));
        }

        let offset_str: String;
        if let Some(o) = offset {
            offset_str = o.to_string();
            params.push(("offset", &offset_str));
        }

        self.request_raw(reqwest::Method::GET, "/products/search", Some(&params)).await
    }

    /// Look up product details by identifier
    ///
    /// # Arguments
    ///
    /// * `identifier` - Product identifier (barcode, ASIN, URL, model number, or ShopSavvy product ID)
    /// * `format` - Optional output format
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let product = client.get_product_details("012345678901", None).await?;
    /// println!("Product: {}", product.data[0].title);
    /// ```
    pub async fn get_product_details(&self, identifier: &str, format: Option<OutputFormat>) -> Result<ApiResponse<Vec<ProductDetails>>> {
        let mut params = vec![("ids", identifier)];

        let format_str;
        if let Some(fmt) = format {
            format_str = fmt.to_string();
            params.push(("format", &format_str));
        }

        self.request(reqwest::Method::GET, "/products", Some(&params), None).await
    }

    /// Look up details for multiple products
    ///
    /// # Arguments
    ///
    /// * `identifiers` - List of product identifiers
    /// * `format` - Optional output format
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let products = client.get_product_details_batch(
    ///     &["012345678901", "B08N5WRWNW"],
    ///     None
    /// ).await?;
    /// ```
    pub async fn get_product_details_batch(&self, identifiers: &[&str], format: Option<OutputFormat>) -> Result<ApiResponse<Vec<ProductDetails>>> {
        let identifiers_str = identifiers.join(",");
        let mut params = vec![("ids", identifiers_str.as_str())];

        let format_str;
        if let Some(fmt) = format {
            format_str = fmt.to_string();
            params.push(("format", &format_str));
        }

        self.request(reqwest::Method::GET, "/products", Some(&params), None).await
    }

    /// Get current offers for a product
    ///
    /// # Arguments
    ///
    /// * `identifier` - Product identifier
    /// * `retailer` - Optional retailer to filter by
    /// * `format` - Optional output format
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = client.get_current_offers("012345678901", None, None).await?;
    /// for product in result.data {
    ///     println!("Product: {}", product.title);
    ///     for offer in product.offers {
    ///         println!("  {}: ${:?}", offer.retailer, offer.price);
    ///     }
    /// }
    /// ```
    pub async fn get_current_offers(&self, identifier: &str, retailer: Option<&str>, format: Option<OutputFormat>) -> Result<ApiResponse<Vec<ProductWithOffers>>> {
        let mut params = vec![("ids", identifier)];

        if let Some(ret) = retailer {
            params.push(("retailer", ret));
        }

        let format_str;
        if let Some(fmt) = format {
            format_str = fmt.to_string();
            params.push(("format", &format_str));
        }

        self.request(reqwest::Method::GET, "/products/offers", Some(&params), None).await
    }

    /// Get current offers for multiple products
    pub async fn get_current_offers_batch(&self, identifiers: &[&str], retailer: Option<&str>, format: Option<OutputFormat>) -> Result<ApiResponse<Vec<ProductWithOffers>>> {
        let identifiers_str = identifiers.join(",");
        let mut params = vec![("ids", identifiers_str.as_str())];

        if let Some(ret) = retailer {
            params.push(("retailer", ret));
        }

        let format_str;
        if let Some(fmt) = format {
            format_str = fmt.to_string();
            params.push(("format", &format_str));
        }

        self.request(reqwest::Method::GET, "/products/offers", Some(&params), None).await
    }

    /// Get price history for a product
    ///
    /// # Arguments
    ///
    /// * `identifier` - Product identifier
    /// * `start_date` - Start date (YYYY-MM-DD format)
    /// * `end_date` - End date (YYYY-MM-DD format)
    /// * `retailer` - Optional retailer to filter by
    /// * `format` - Optional output format
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let history = client.get_price_history(
    ///     "012345678901",
    ///     "2024-01-01",
    ///     "2024-01-31",
    ///     None,
    ///     None
    /// ).await?;
    /// ```
    pub async fn get_price_history(&self, identifier: &str, start_date: &str, end_date: &str, retailer: Option<&str>, format: Option<OutputFormat>) -> Result<ApiResponse<Vec<OfferWithHistory>>> {
        let mut params = vec![
            ("ids", identifier),
            ("start_date", start_date),
            ("end_date", end_date),
        ];

        if let Some(ret) = retailer {
            params.push(("retailer", ret));
        }

        let format_str;
        if let Some(fmt) = format {
            format_str = fmt.to_string();
            params.push(("format", &format_str));
        }

        self.request(reqwest::Method::GET, "/products/offers/history", Some(&params), None).await
    }

    /// Schedule product monitoring
    ///
    /// # Arguments
    ///
    /// * `identifier` - Product identifier
    /// * `frequency` - How often to refresh
    /// * `retailer` - Optional retailer to monitor
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = client.schedule_product_monitoring(
    ///     "012345678901",
    ///     MonitoringFrequency::Daily,
    ///     None
    /// ).await?;
    /// ```
    pub async fn schedule_product_monitoring(&self, identifier: &str, frequency: MonitoringFrequency, retailer: Option<&str>) -> Result<ApiResponse<ScheduleResponse>> {
        let mut body = serde_json::json!({
            "identifier": identifier,
            "frequency": frequency.to_string(),
        });

        if let Some(ret) = retailer {
            body["retailer"] = serde_json::Value::String(ret.to_string());
        }

        self.request(reqwest::Method::POST, "/products/schedule", None, Some(&body)).await
    }

    /// Schedule monitoring for multiple products
    pub async fn schedule_product_monitoring_batch(&self, identifiers: &[&str], frequency: MonitoringFrequency, retailer: Option<&str>) -> Result<ApiResponse<Vec<ScheduleBatchResponse>>> {
        let identifiers_str = identifiers.join(",");
        let mut body = serde_json::json!({
            "identifiers": identifiers_str,
            "frequency": frequency.to_string(),
        });

        if let Some(ret) = retailer {
            body["retailer"] = serde_json::Value::String(ret.to_string());
        }

        self.request(reqwest::Method::POST, "/products/schedule", None, Some(&body)).await
    }

    /// Get all scheduled products
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let scheduled = client.get_scheduled_products().await?;
    /// println!("Monitoring {} products", scheduled.data.len());
    /// ```
    pub async fn get_scheduled_products(&self) -> Result<ApiResponse<Vec<ScheduledProduct>>> {
        self.request(reqwest::Method::GET, "/products/scheduled", None, None).await
    }

    /// Remove product from monitoring schedule
    pub async fn remove_product_from_schedule(&self, identifier: &str) -> Result<ApiResponse<RemoveResponse>> {
        let body = serde_json::json!({
            "identifier": identifier,
        });

        self.request(reqwest::Method::DELETE, "/products/schedule", None, Some(&body)).await
    }

    /// Remove multiple products from monitoring schedule
    pub async fn remove_products_from_schedule(&self, identifiers: &[&str]) -> Result<ApiResponse<Vec<RemoveBatchResponse>>> {
        let identifiers_str = identifiers.join(",");
        let body = serde_json::json!({
            "identifiers": identifiers_str,
        });

        self.request(reqwest::Method::DELETE, "/products/schedule", None, Some(&body)).await
    }

    /// Get API usage information
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let usage = client.get_usage().await?;
    /// println!("Credits remaining: {}", usage.data.current_period.credits_remaining);
    /// ```
    pub async fn get_usage(&self) -> Result<ApiResponse<UsageInfo>> {
        self.request(reqwest::Method::GET, "/usage", None, None).await
    }
}
