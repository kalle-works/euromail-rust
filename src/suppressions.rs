use serde::Serialize;

use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{
    ImportSuppressionsParams, ImportSuppressionsResult, ListParams, PaginatedResponse,
    Suppression,
};

#[derive(Serialize)]
struct AddSuppressionBody {
    email_address: String,
    reason: String,
}

impl EuroMail {
    /// Add an email address to the suppression list.
    ///
    /// Suppressed addresses will be silently skipped when sending. The `reason`
    /// defaults to `"manual"` if not provided.
    pub async fn add_suppression(
        &self,
        email: &str,
        reason: Option<&str>,
    ) -> Result<Suppression, EuroMailError> {
        let body = AddSuppressionBody {
            email_address: email.to_string(),
            reason: reason.unwrap_or("manual").to_string(),
        };
        self.post("/v1/suppressions", &body).await
    }

    /// Remove an email address from the suppression list, allowing delivery again.
    pub async fn delete_suppression(&self, email: &str) -> Result<(), EuroMailError> {
        let encoded = email.replace('@', "%40");
        self.delete(&format!("/v1/suppressions/{encoded}")).await
    }

    /// List all suppressed email addresses with optional pagination.
    pub async fn list_suppressions(
        &self,
        params: Option<&ListParams>,
    ) -> Result<PaginatedResponse<Suppression>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            if let Some(page) = p.page {
                query.push(("page", page.to_string()));
            }
            if let Some(per_page) = p.per_page {
                query.push(("per_page", per_page.to_string()));
            }
        }
        self.get_with_query("/v1/suppressions", &query).await
    }

    /// Bulk-import addresses onto the suppression list (up to 10,000 per request).
    ///
    /// Invalid addresses are reported back rather than causing the whole request
    /// to fail; check `invalid_addresses` on the result. Already-suppressed
    /// addresses are silently skipped and not counted in `inserted`.
    pub async fn import_suppressions(
        &self,
        params: &ImportSuppressionsParams,
    ) -> Result<ImportSuppressionsResult, EuroMailError> {
        self.post("/v1/suppressions/import", params).await
    }

    /// Export the entire suppression list as CSV.
    pub async fn export_suppressions(&self) -> Result<String, EuroMailError> {
        self.get_raw("/v1/suppressions/export", &[]).await
    }
}
