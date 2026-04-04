use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::Account;

impl EuroMail {
    /// Get the account associated with the current API key.
    pub async fn get_account(&self) -> Result<Account, EuroMailError> {
        self.get("/v1/account").await
    }

    /// Export all account data as a JSON string (for backup or migration).
    pub async fn export_account(&self) -> Result<String, EuroMailError> {
        self.get_raw("/v1/account/export", &[]).await
    }

    /// Permanently delete the account and all associated data.
    ///
    /// **This action is irreversible.**
    pub async fn delete_account(&self) -> Result<(), EuroMailError> {
        self.delete("/v1/account").await
    }
}
