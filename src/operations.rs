use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{ListParams, Operation, PaginatedResponse};

impl EuroMail {
    /// List async operations (broadcasts, newsletter sends, bulk imports).
    pub async fn list_operations(
        &self,
        params: Option<&ListParams>,
    ) -> Result<PaginatedResponse<Operation>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            if let Some(page) = p.page {
                query.push(("page", page.to_string()));
            }
            if let Some(per_page) = p.per_page {
                query.push(("per_page", per_page.to_string()));
            }
        }
        self.get_with_query("/v1/operations", &query).await
    }

    /// Get a specific operation by ID.
    pub async fn get_operation(&self, id: &str) -> Result<Operation, EuroMailError> {
        self.get(&format!("/v1/operations/{id}")).await
    }
}
