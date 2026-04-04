use serde::Serialize;

use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::EmailValidation;

#[derive(Serialize)]
struct ValidateBody {
    email: String,
}

impl EuroMail {
    /// Validate an email address (syntax, MX, disposable, role, free provider).
    pub async fn validate_email(&self, email: &str) -> Result<EmailValidation, EuroMailError> {
        let body = ValidateBody {
            email: email.to_string(),
        };
        self.post_direct("/v1/validate", &body).await
    }
}
