use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{
    CreateNewsletterParams, Newsletter, NewsletterSendResponse, UpdateNewsletterParams,
};

impl EuroMail {
    /// Create a new newsletter draft.
    pub async fn create_newsletter(
        &self,
        params: &CreateNewsletterParams,
    ) -> Result<Newsletter, EuroMailError> {
        self.post("/v1/newsletters", params).await
    }

    /// Get a newsletter by ID.
    pub async fn get_newsletter(&self, id: &str) -> Result<Newsletter, EuroMailError> {
        self.get(&format!("/v1/newsletters/{id}")).await
    }

    /// Update a newsletter draft.
    pub async fn update_newsletter(
        &self,
        id: &str,
        params: &UpdateNewsletterParams,
    ) -> Result<Newsletter, EuroMailError> {
        self.put(&format!("/v1/newsletters/{id}"), params).await
    }

    /// Delete a newsletter.
    pub async fn delete_newsletter(&self, id: &str) -> Result<(), EuroMailError> {
        self.delete(&format!("/v1/newsletters/{id}")).await
    }

    /// List newsletters with optional limit and offset.
    pub async fn list_newsletters(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Newsletter>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(l) = limit {
            query.push(("limit", l.to_string()));
        }
        if let Some(o) = offset {
            query.push(("offset", o.to_string()));
        }
        #[derive(serde::Deserialize)]
        struct Resp {
            data: Vec<Newsletter>,
        }
        let resp: Resp = self.get_with_query("/v1/newsletters", &query).await?;
        Ok(resp.data)
    }

    /// Send a newsletter to its contact list.
    pub async fn send_newsletter(&self, id: &str) -> Result<NewsletterSendResponse, EuroMailError> {
        self.post(&format!("/v1/newsletters/{id}/send"), &()).await
    }
}
