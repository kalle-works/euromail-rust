use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::errors::{ApiErrorBody, EuroMailError};

const DEFAULT_BASE_URL: &str = "https://api.euromail.dev";

/// Client for the EuroMail API.
///
/// All API methods are implemented as `async` methods on this struct. Create an
/// instance with [`EuroMail::new`] and an API key, then call any method to
/// interact with the EuroMail service.
///
/// # Example
///
/// ```rust,no_run
/// use euromail::EuroMail;
///
/// # async fn run() -> Result<(), euromail::EuroMailError> {
/// let client = EuroMail::new("em_live_your_api_key");
/// let account = client.get_account().await?;
/// println!("Account: {} ({})", account.name, account.plan);
/// # Ok(())
/// # }
/// ```
pub struct EuroMail {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
}

#[derive(Deserialize)]
pub(crate) struct DataEnvelope<T> {
    pub data: T,
}

impl EuroMail {
    /// Create a new client with the given API key.
    ///
    /// The base URL defaults to `https://api.euromail.dev` and can be
    /// overridden by setting the `EUROMAIL_API_URL` environment variable.
    pub fn new(api_key: impl Into<String>) -> Self {
        let base_url =
            std::env::var("EUROMAIL_API_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        Self::with_base_url(api_key, base_url)
    }

    /// Create a new client with an explicit base URL.
    ///
    /// Useful for testing against a local or staging server.
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        let api_key = api_key.into();
        let base_url_str = base_url.into();
        if !base_url_str.starts_with("https://")
            && !base_url_str.starts_with("http://localhost")
            && !base_url_str.starts_with("http://127.0.0.1")
        {
            eprintln!(
                "WARNING: EuroMail base URL does not use HTTPS. API keys will be sent in cleartext."
            );
        }
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_key}"))
                .expect("invalid api key characters"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");

        Self {
            http,
            base_url: base_url_str.trim_end_matches('/').to_string(),
        }
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, EuroMailError> {
        let resp = self
            .http
            .get(format!("{}{path}", self.base_url))
            .send()
            .await?;
        let resp = Self::check_status(resp).await?;
        let envelope: DataEnvelope<T> = resp.json().await?;
        Ok(envelope.data)
    }

    pub(crate) async fn get_direct<T: DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, EuroMailError> {
        let resp = self
            .http
            .get(format!("{}{path}", self.base_url))
            .send()
            .await?;
        let resp = Self::check_status(resp).await?;
        Ok(resp.json().await?)
    }

    pub(crate) async fn get_with_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, EuroMailError> {
        let resp = self
            .http
            .get(format!("{}{path}", self.base_url))
            .query(query)
            .send()
            .await?;
        let resp = Self::check_status(resp).await?;
        Ok(resp.json().await?)
    }

    pub(crate) async fn get_raw(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<String, EuroMailError> {
        let resp = self
            .http
            .get(format!("{}{path}", self.base_url))
            .query(query)
            .send()
            .await?;
        let resp = Self::check_status(resp).await?;
        Ok(resp.text().await?)
    }

    pub(crate) async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, EuroMailError> {
        let resp = self
            .http
            .post(format!("{}{path}", self.base_url))
            .json(body)
            .send()
            .await?;
        let resp = Self::check_status(resp).await?;
        let envelope: DataEnvelope<T> = resp.json().await?;
        Ok(envelope.data)
    }

    pub(crate) async fn post_direct<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, EuroMailError> {
        let resp = self
            .http
            .post(format!("{}{path}", self.base_url))
            .json(body)
            .send()
            .await?;
        let resp = Self::check_status(resp).await?;
        Ok(resp.json().await?)
    }

    pub(crate) async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, EuroMailError> {
        let resp = self
            .http
            .put(format!("{}{path}", self.base_url))
            .json(body)
            .send()
            .await?;
        let resp = Self::check_status(resp).await?;
        let envelope: DataEnvelope<T> = resp.json().await?;
        Ok(envelope.data)
    }

    pub(crate) async fn put_direct<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, EuroMailError> {
        let resp = self
            .http
            .put(format!("{}{path}", self.base_url))
            .json(body)
            .send()
            .await?;
        let resp = Self::check_status(resp).await?;
        Ok(resp.json().await?)
    }

    pub(crate) async fn patch<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, EuroMailError> {
        let resp = self
            .http
            .patch(format!("{}{path}", self.base_url))
            .json(body)
            .send()
            .await?;
        let resp = Self::check_status(resp).await?;
        let envelope: DataEnvelope<T> = resp.json().await?;
        Ok(envelope.data)
    }

    pub(crate) async fn delete_with_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, EuroMailError> {
        let resp = self
            .http
            .delete(format!("{}{path}", self.base_url))
            .query(query)
            .send()
            .await?;
        let resp = Self::check_status(resp).await?;
        Ok(resp.json().await?)
    }

    pub(crate) async fn delete(&self, path: &str) -> Result<(), EuroMailError> {
        let resp = self
            .http
            .delete(format!("{}{path}", self.base_url))
            .send()
            .await?;
        Self::check_status(resp).await?;
        Ok(())
    }

    async fn check_status(resp: reqwest::Response) -> Result<reqwest::Response, EuroMailError> {
        let status = resp.status().as_u16();
        if status < 400 {
            return Ok(resp);
        }

        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let body: ApiErrorBody = resp.json().await.unwrap_or(ApiErrorBody {
            code: "unknown".to_string(),
            message: "Unknown error".to_string(),
        });

        Err(match status {
            401 => EuroMailError::Authentication(body.message),
            404 => EuroMailError::NotFound(body.message),
            422 => EuroMailError::Validation {
                code: body.code,
                message: body.message,
            },
            429 => EuroMailError::RateLimit {
                retry_after,
                message: body.message,
            },
            _ => EuroMailError::Api {
                status,
                code: body.code,
                message: body.message,
            },
        })
    }
}
