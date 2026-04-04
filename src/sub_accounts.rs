use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{
    AnalyticsOverviewResponse, CreateApiKeyParams, CreateApiKeyResponse, CreateSubAccountParams,
    PaginatedResponse, SubAccount, UpdateSubAccountParams,
};

impl EuroMail {
    /// Create a new sub-account under the current (parent) account.
    ///
    /// Requires the `sub_accounts:manage` scope and a plan that supports sub-accounts
    /// (business or enterprise).
    pub async fn create_sub_account(
        &self,
        params: &CreateSubAccountParams,
    ) -> Result<SubAccount, EuroMailError> {
        self.post("/v1/accounts", params).await
    }

    /// List all sub-accounts belonging to the current account.
    pub async fn list_sub_accounts(
        &self,
        page: Option<i64>,
        per_page: Option<i64>,
    ) -> Result<PaginatedResponse<SubAccount>, EuroMailError> {
        let mut query = Vec::new();
        if let Some(p) = page {
            query.push(("page", p.to_string()));
        }
        if let Some(pp) = per_page {
            query.push(("per_page", pp.to_string()));
        }
        self.get_with_query("/v1/accounts", &query).await
    }

    /// Get a specific sub-account by ID.
    pub async fn get_sub_account(&self, id: &str) -> Result<SubAccount, EuroMailError> {
        self.get(&format!("/v1/accounts/{id}")).await
    }

    /// Update a sub-account's name, quota, or active status.
    pub async fn update_sub_account(
        &self,
        id: &str,
        params: &UpdateSubAccountParams,
    ) -> Result<SubAccount, EuroMailError> {
        self.patch(&format!("/v1/accounts/{id}"), params).await
    }

    /// Delete a sub-account and all its data.
    pub async fn delete_sub_account(&self, id: &str) -> Result<(), EuroMailError> {
        self.delete(&format!("/v1/accounts/{id}")).await
    }

    /// Get analytics for a specific sub-account.
    pub async fn get_sub_account_analytics(
        &self,
        id: &str,
        period: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<AnalyticsOverviewResponse, EuroMailError> {
        let mut query = Vec::new();
        if let Some(p) = period {
            query.push(("period", p.to_string()));
        }
        if let Some(f) = from {
            query.push(("from", f.to_string()));
        }
        if let Some(t) = to {
            query.push(("to", t.to_string()));
        }
        self.get_with_query(&format!("/v1/accounts/{id}/analytics"), &query)
            .await
    }

    /// Get aggregate analytics across the parent account and all sub-accounts.
    pub async fn get_aggregate_analytics(
        &self,
        period: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<AnalyticsOverviewResponse, EuroMailError> {
        let mut query = Vec::new();
        if let Some(p) = period {
            query.push(("period", p.to_string()));
        }
        if let Some(f) = from {
            query.push(("from", f.to_string()));
        }
        if let Some(t) = to {
            query.push(("to", t.to_string()));
        }
        self.get_with_query("/v1/analytics/aggregate", &query).await
    }

    /// Create an API key for a sub-account.
    pub async fn create_sub_account_api_key(
        &self,
        id: &str,
        params: &CreateApiKeyParams,
    ) -> Result<CreateApiKeyResponse, EuroMailError> {
        self.post(&format!("/v1/accounts/{id}/api-keys"), params)
            .await
    }
}
