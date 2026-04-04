use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{
    CreateTemplateParams, ListParams, PaginatedResponse, Template, UpdateTemplateParams,
};

impl EuroMail {
    /// Create a new email template.
    pub async fn create_template(
        &self,
        params: &CreateTemplateParams,
    ) -> Result<Template, EuroMailError> {
        self.post("/v1/templates", params).await
    }

    /// Get a template by ID.
    pub async fn get_template(&self, template_id: &str) -> Result<Template, EuroMailError> {
        self.get(&format!("/v1/templates/{template_id}")).await
    }

    /// Update an existing template. Only provided fields are changed.
    pub async fn update_template(
        &self,
        template_id: &str,
        params: &UpdateTemplateParams,
    ) -> Result<Template, EuroMailError> {
        self.put(&format!("/v1/templates/{template_id}"), params)
            .await
    }

    /// Delete a template.
    pub async fn delete_template(&self, template_id: &str) -> Result<(), EuroMailError> {
        self.delete(&format!("/v1/templates/{template_id}")).await
    }

    /// List all templates with optional pagination.
    pub async fn list_templates(
        &self,
        params: Option<&ListParams>,
    ) -> Result<PaginatedResponse<Template>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            if let Some(page) = p.page {
                query.push(("page", page.to_string()));
            }
            if let Some(per_page) = p.per_page {
                query.push(("per_page", per_page.to_string()));
            }
        }
        self.get_with_query("/v1/templates", &query).await
    }
}
