use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{
    BroadcastParams, BroadcastResponse, Email, EmailDetail, LinkClickStat, ListParams,
    PaginatedResponse, SendBatchParams, SendBatchResponse, SendEmailParams, SendEmailResponse,
};

impl EuroMail {
    /// Send a single transactional email.
    ///
    /// Returns the queued email with its ID and message ID for tracking.
    ///
    /// # Errors
    ///
    /// Returns [`EuroMailError::Validation`] if the sender domain is not verified
    /// or required fields are missing.
    pub async fn send_email(
        &self,
        params: &SendEmailParams,
    ) -> Result<SendEmailResponse, EuroMailError> {
        self.post("/v1/emails", params).await
    }

    /// Send up to 100 emails in a single request.
    ///
    /// Returns both successful sends and per-email errors. Partial success is
    /// possible — check both `data` and `errors` in the response.
    pub async fn send_batch(
        &self,
        params: &SendBatchParams,
    ) -> Result<SendBatchResponse, EuroMailError> {
        self.post_direct("/v1/emails/batch", params).await
    }

    /// Retrieve a single email with its full delivery event history.
    pub async fn get_email(&self, email_id: &str) -> Result<EmailDetail, EuroMailError> {
        self.get(&format!("/v1/emails/{email_id}")).await
    }

    /// Cancel a scheduled or queued email before it is sent.
    ///
    /// Only emails in `queued` or `scheduled` status can be cancelled.
    pub async fn cancel_email(&self, email_id: &str) -> Result<SendEmailResponse, EuroMailError> {
        self.post(&format!("/v1/emails/{email_id}/cancel"), &())
            .await
    }

    /// Send an email to all active subscribers in a contact list.
    pub async fn send_broadcast(
        &self,
        params: &BroadcastParams,
    ) -> Result<BroadcastResponse, EuroMailError> {
        self.post("/v1/emails/broadcast", params).await
    }

    /// Retrieve per-link click statistics for a sent email.
    ///
    /// Returns click counts for every tracked URL in the email body.
    pub async fn get_email_links(
        &self,
        email_id: &str,
    ) -> Result<Vec<LinkClickStat>, EuroMailError> {
        self.get(&format!("/v1/emails/{email_id}/links")).await
    }

    /// List emails with optional pagination and status filter.
    pub async fn list_emails(
        &self,
        params: Option<&ListParams>,
        status: Option<&str>,
    ) -> Result<PaginatedResponse<Email>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            if let Some(page) = p.page {
                query.push(("page", page.to_string()));
            }
            if let Some(per_page) = p.per_page {
                query.push(("per_page", per_page.to_string()));
            }
        }
        if let Some(s) = status {
            query.push(("status", s.to_string()));
        }
        self.get_with_query("/v1/emails", &query).await
    }
}
