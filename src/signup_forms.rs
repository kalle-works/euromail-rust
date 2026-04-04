use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{CreateSignupFormParams, SignupForm, UpdateSignupFormParams};

impl EuroMail {
    /// Create a new signup form.
    pub async fn create_signup_form(
        &self,
        params: &CreateSignupFormParams,
    ) -> Result<SignupForm, EuroMailError> {
        self.post("/v1/signup-forms", params).await
    }

    /// List all signup forms in the account.
    pub async fn list_signup_forms(&self) -> Result<Vec<SignupForm>, EuroMailError> {
        #[derive(serde::Deserialize)]
        struct Resp {
            data: Vec<SignupForm>,
        }
        let resp: Resp = self.get_direct("/v1/signup-forms").await?;
        Ok(resp.data)
    }

    /// Get a signup form by ID.
    pub async fn get_signup_form(&self, id: &str) -> Result<SignupForm, EuroMailError> {
        self.get(&format!("/v1/signup-forms/{id}")).await
    }

    /// Update a signup form.
    pub async fn update_signup_form(
        &self,
        id: &str,
        params: &UpdateSignupFormParams,
    ) -> Result<SignupForm, EuroMailError> {
        self.put(&format!("/v1/signup-forms/{id}"), params).await
    }

    /// Delete a signup form.
    pub async fn delete_signup_form(&self, id: &str) -> Result<(), EuroMailError> {
        self.delete(&format!("/v1/signup-forms/{id}")).await
    }

    /// Toggle a signup form's active state.
    pub async fn toggle_signup_form(&self, id: &str) -> Result<SignupForm, EuroMailError> {
        self.post(&format!("/v1/signup-forms/{id}/toggle"), &())
            .await
    }
}
