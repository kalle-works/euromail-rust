use serde::Deserialize;

/// Errors returned by the EuroMail SDK.
///
/// All API methods return `Result<T, EuroMailError>`. HTTP-level errors from the
/// EuroMail API are mapped to specific variants based on status code, while
/// network and deserialization failures surface as [`EuroMailError::Http`].
///
/// # Example
///
/// ```rust,no_run
/// # use euromail::{EuroMail, EuroMailError};
/// # async fn run() -> Result<(), EuroMailError> {
/// let client = EuroMail::new("em_live_key");
/// match client.get_email("nonexistent").await {
///     Err(EuroMailError::NotFound(msg)) => eprintln!("Not found: {msg}"),
///     Err(EuroMailError::RateLimit { retry_after, .. }) => {
///         eprintln!("Rate limited, retry after {retry_after:?}s");
///     }
///     Err(e) => eprintln!("Error: {e}"),
///     Ok(detail) => println!("Email: {}", detail.email.id),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum EuroMailError {
    /// Invalid or expired API key (HTTP 401).
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Request failed validation — e.g. missing required fields (HTTP 422).
    #[error("Validation error [{code}]: {message}")]
    Validation { code: String, message: String },

    /// Too many requests. `retry_after` contains the suggested wait in seconds
    /// if the server provided a `Retry-After` header (HTTP 429).
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        retry_after: Option<u64>,
        message: String,
    },

    /// The requested resource does not exist (HTTP 404).
    #[error("Not found: {0}")]
    NotFound(String),

    /// Any other API error (HTTP 4xx/5xx).
    #[error("API error [{status}] {code}: {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
    },

    /// Network or deserialization error from the underlying HTTP client.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Deserialize)]
pub(crate) struct ApiErrorBody {
    #[serde(default = "default_code")]
    pub code: String,
    #[serde(default = "default_message")]
    pub message: String,
}

fn default_code() -> String {
    "unknown".to_string()
}

fn default_message() -> String {
    "Unknown error".to_string()
}
