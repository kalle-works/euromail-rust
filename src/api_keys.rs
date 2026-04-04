use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{ApiKey, CreateApiKeyParams, CreateApiKeyResponse};

impl EuroMail {
    /// Create a new API key with optional scoped permissions.
    ///
    /// The full key is returned only in this response — store it securely.
    pub async fn create_api_key(
        &self,
        params: &CreateApiKeyParams,
    ) -> Result<CreateApiKeyResponse, EuroMailError> {
        self.post("/v1/api-keys", params).await
    }

    /// List all API keys (without secret portions).
    pub async fn list_api_keys(&self) -> Result<Vec<ApiKey>, EuroMailError> {
        self.get("/v1/api-keys").await
    }

    /// Permanently delete an API key.
    pub async fn delete_api_key(&self, id: &str) -> Result<(), EuroMailError> {
        self.delete(&format!("/v1/api-keys/{id}")).await
    }
}
