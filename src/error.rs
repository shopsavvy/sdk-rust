use thiserror::Error;

/// Result type alias for ShopSavvy API operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for ShopSavvy API operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("Authentication failed: {message}")]
    Authentication { message: String, status_code: u16 },

    #[error("Resource not found: {message}")]
    NotFound { message: String, status_code: u16 },

    #[error("Validation error: {message}")]
    Validation { message: String, status_code: u16 },

    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String, status_code: u16 },

    #[error("API error ({status_code}): {message}")]
    Api { message: String, status_code: u16 },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid API key format. API keys should start with ss_live_ or ss_test_")]
    InvalidApiKey,

    #[error("API key is required. Get one at https://shopsavvy.com/data")]
    MissingApiKey,

    #[error("Request timeout")]
    Timeout,
}

impl Error {
    pub(crate) fn from_status_code(status_code: u16, message: String) -> Self {
        match status_code {
            401 => Error::Authentication {
                message: "Authentication failed. Check your API key.".to_string(),
                status_code,
            },
            404 => Error::NotFound {
                message: "Resource not found".to_string(),
                status_code,
            },
            422 => Error::Validation {
                message: "Request validation failed. Check your parameters.".to_string(),
                status_code,
            },
            429 => Error::RateLimit {
                message: "Rate limit exceeded. Please slow down your requests.".to_string(),
                status_code,
            },
            _ => Error::Api {
                message,
                status_code,
            },
        }
    }
}