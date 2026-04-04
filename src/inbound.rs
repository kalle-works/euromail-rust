use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{InboundEmail, ListParams, PaginatedResponse};

impl EuroMail {
    /// List received inbound emails with optional pagination.
    pub async fn list_inbound_emails(
        &self,
        params: Option<&ListParams>,
    ) -> Result<PaginatedResponse<InboundEmail>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            if let Some(page) = p.page {
                query.push(("page", page.to_string()));
            }
            if let Some(per_page) = p.per_page {
                query.push(("per_page", per_page.to_string()));
            }
        }
        self.get_with_query("/v1/inbound", &query).await
    }

    /// Get a single inbound email by ID.
    pub async fn get_inbound_email(&self, id: &str) -> Result<InboundEmail, EuroMailError> {
        self.get(&format!("/v1/inbound/{id}")).await
    }

    /// Delete an inbound email.
    pub async fn delete_inbound_email(&self, id: &str) -> Result<(), EuroMailError> {
        self.delete(&format!("/v1/inbound/{id}")).await
    }
}
