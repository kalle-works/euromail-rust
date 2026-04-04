use crate::client::{DataEnvelope, EuroMail};
use crate::errors::EuroMailError;
use crate::types::DeadLetter;

impl EuroMail {
    /// List emails that permanently failed delivery after all retry attempts.
    ///
    /// Use `count` to limit the number of results (defaults to all).
    pub async fn list_dead_letters(
        &self,
        count: Option<i64>,
    ) -> Result<Vec<DeadLetter>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(c) = count {
            query.push(("count", c.to_string()));
        }
        let envelope: DataEnvelope<Vec<DeadLetter>> =
            self.get_with_query("/v1/dead-letters", &query).await?;
        Ok(envelope.data)
    }

    /// Retry delivery of a dead letter email.
    ///
    /// The email is re-queued for delivery with a fresh retry counter.
    pub async fn retry_dead_letter(&self, id: &str) -> Result<(), EuroMailError> {
        let _: serde_json::Value = self
            .post_direct(
                &format!("/v1/dead-letters/{id}/retry"),
                &serde_json::json!({}),
            )
            .await?;
        Ok(())
    }

    /// Permanently delete a dead letter entry.
    pub async fn delete_dead_letter(&self, id: &str) -> Result<(), EuroMailError> {
        self.delete(&format!("/v1/dead-letters/{id}")).await
    }
}
