use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{GdprEraseResponse, GdprExportResponse};

impl EuroMail {
    /// Export all personal data associated with an email address (GDPR Art. 15).
    ///
    /// Returns emails, events, suppressions, unsubscribe events, and inbound
    /// emails linked to the given address.
    pub async fn gdpr_export_email(
        &self,
        email: &str,
    ) -> Result<GdprExportResponse, EuroMailError> {
        self.get_with_query("/v1/gdpr/export", &[("email", email.to_string())])
            .await
    }

    /// Erase all personal data associated with an email address (GDPR Art. 17).
    ///
    /// **This action is irreversible.** All emails, events, and related records
    /// for the given address are permanently deleted.
    pub async fn gdpr_erase_email(&self, email: &str) -> Result<GdprEraseResponse, EuroMailError> {
        self.delete_with_query("/v1/gdpr/erase", &[("email", email.to_string())])
            .await
    }
}
