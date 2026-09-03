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

/// The EuroMail API wraps error details in a nested `error` object:
/// `{"error": {"type": "...", "code": "...", "message": "..."}}`. This
/// mirrors that shape while also accepting a flat body (`{"code", "message"}`)
/// for forward compatibility, since some endpoints or future API versions may
/// not nest it.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ApiErrorBody {
    #[serde(default)]
    pub error: Option<ApiErrorDetail>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(rename = "type", default)]
    pub error_type: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ApiErrorDetail {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(rename = "type", default)]
    pub error_type: Option<String>,
}

impl ApiErrorBody {
    /// Resolve to `(code, message, type)`, preferring the nested `error`
    /// object when present and falling back to a flat body, then finally to
    /// generic placeholders if the response body didn't parse as JSON at all
    /// or carried neither shape.
    pub(crate) fn resolve(self) -> (String, String, Option<String>) {
        let nested = self.error;
        let code = nested
            .as_ref()
            .and_then(|e| e.code.clone())
            .or(self.code)
            .unwrap_or_else(|| "unknown".to_string());
        let message = nested
            .as_ref()
            .and_then(|e| e.message.clone())
            .or(self.message)
            .unwrap_or_else(|| "Unknown error".to_string());
        let error_type = nested
            .as_ref()
            .and_then(|e| e.error_type.clone())
            .or(self.error_type);
        (code, message, error_type)
    }
}
