use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{
    CreateInboundRouteParams, InboundRoute, ListParams, PaginatedResponse, UpdateInboundRouteParams,
};

impl EuroMail {
    /// Create a new inbound routing rule.
    ///
    /// Routes match incoming emails by pattern and forward them to a webhook URL.
    pub async fn create_inbound_route(
        &self,
        params: &CreateInboundRouteParams,
    ) -> Result<InboundRoute, EuroMailError> {
        self.post("/v1/inbound-routes", params).await
    }

    /// List all inbound routes with optional pagination.
    pub async fn list_inbound_routes(
        &self,
        params: Option<&ListParams>,
    ) -> Result<PaginatedResponse<InboundRoute>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            if let Some(page) = p.page {
                query.push(("page", page.to_string()));
            }
            if let Some(per_page) = p.per_page {
                query.push(("per_page", per_page.to_string()));
            }
        }
        self.get_with_query("/v1/inbound-routes", &query).await
    }

    /// Get an inbound route by ID.
    pub async fn get_inbound_route(&self, id: &str) -> Result<InboundRoute, EuroMailError> {
        self.get(&format!("/v1/inbound-routes/{id}")).await
    }

    /// Update an inbound route. Only provided fields are changed.
    pub async fn update_inbound_route(
        &self,
        id: &str,
        params: &UpdateInboundRouteParams,
    ) -> Result<InboundRoute, EuroMailError> {
        self.put(&format!("/v1/inbound-routes/{id}"), params).await
    }

    /// Delete an inbound route.
    pub async fn delete_inbound_route(&self, id: &str) -> Result<(), EuroMailError> {
        self.delete(&format!("/v1/inbound-routes/{id}")).await
    }
}
