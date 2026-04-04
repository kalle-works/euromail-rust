use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{AuditLog, ListParams, PaginatedResponse};

impl EuroMail {
    /// List audit log entries with optional pagination.
    ///
    /// Audit logs record account actions such as domain creation, API key
    /// changes, and webhook modifications.
    pub async fn list_audit_logs(
        &self,
        params: Option<&ListParams>,
    ) -> Result<PaginatedResponse<AuditLog>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            if let Some(page) = p.page {
                query.push(("page", page.to_string()));
            }
            if let Some(per_page) = p.per_page {
                query.push(("per_page", per_page.to_string()));
            }
        }
        self.get_with_query("/v1/audit-logs", &query).await
    }
}
