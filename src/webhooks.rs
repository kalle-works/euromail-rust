use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{
    CreateWebhookParams, ListParams, PaginatedResponse, UpdateWebhookParams, Webhook,
    WebhookTestResponse,
};

impl EuroMail {
    /// Create a new webhook endpoint.
    pub async fn create_webhook(
        &self,
        params: &CreateWebhookParams,
    ) -> Result<Webhook, EuroMailError> {
        self.post("/v1/webhooks", params).await
    }

    /// Get a webhook by ID.
    pub async fn get_webhook(&self, webhook_id: &str) -> Result<Webhook, EuroMailError> {
        self.get(&format!("/v1/webhooks/{webhook_id}")).await
    }

    /// Update a webhook's URL, events, or active status.
    pub async fn update_webhook(
        &self,
        webhook_id: &str,
        params: &UpdateWebhookParams,
    ) -> Result<Webhook, EuroMailError> {
        self.put(&format!("/v1/webhooks/{webhook_id}"), params)
            .await
    }

    /// Delete a webhook.
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<(), EuroMailError> {
        self.delete(&format!("/v1/webhooks/{webhook_id}")).await
    }

    /// List all webhooks with optional pagination.
    pub async fn list_webhooks(
        &self,
        params: Option<&ListParams>,
    ) -> Result<PaginatedResponse<Webhook>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            if let Some(page) = p.page {
                query.push(("page", page.to_string()));
            }
            if let Some(per_page) = p.per_page {
                query.push(("per_page", per_page.to_string()));
            }
        }
        self.get_with_query("/v1/webhooks", &query).await
    }

    /// Send a test event to a webhook endpoint to verify it's working.
    pub async fn test_webhook(
        &self,
        webhook_id: &str,
    ) -> Result<WebhookTestResponse, EuroMailError> {
        self.post(
            &format!("/v1/webhooks/{webhook_id}/test"),
            &serde_json::json!({}),
        )
        .await
    }
}
