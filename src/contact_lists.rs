use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{
    AddContactParams, BulkAddContactsParams, BulkAddContactsResponse, Contact, ContactList,
    CreateContactListParams, ListContactsParams, PaginatedResponse, UpdateContactListParams,
};

impl EuroMail {
    /// Create a new contact list.
    pub async fn create_contact_list(
        &self,
        params: &CreateContactListParams,
    ) -> Result<ContactList, EuroMailError> {
        self.post("/v1/contact-lists", params).await
    }

    /// Get a contact list by ID.
    pub async fn get_contact_list(&self, list_id: &str) -> Result<ContactList, EuroMailError> {
        self.get(&format!("/v1/contact-lists/{list_id}")).await
    }

    /// Update a contact list's name, description, or double opt-in setting.
    pub async fn update_contact_list(
        &self,
        list_id: &str,
        params: &UpdateContactListParams,
    ) -> Result<ContactList, EuroMailError> {
        self.put(&format!("/v1/contact-lists/{list_id}"), params)
            .await
    }

    /// Delete a contact list and all its contacts.
    pub async fn delete_contact_list(&self, list_id: &str) -> Result<(), EuroMailError> {
        self.delete(&format!("/v1/contact-lists/{list_id}")).await
    }

    /// List all contact lists in the account.
    pub async fn list_contact_lists(&self) -> Result<Vec<ContactList>, EuroMailError> {
        #[derive(serde::Deserialize)]
        struct Resp {
            data: Vec<ContactList>,
        }
        let resp: Resp = self.get_direct("/v1/contact-lists").await?;
        Ok(resp.data)
    }

    /// Add a single contact to a list.
    pub async fn add_contact(
        &self,
        list_id: &str,
        params: &AddContactParams,
    ) -> Result<Contact, EuroMailError> {
        self.post(&format!("/v1/contact-lists/{list_id}/contacts"), params)
            .await
    }

    /// Add multiple contacts to a list in a single request.
    ///
    /// Duplicates are skipped. Check [`BulkAddContactsResponse::inserted`] for
    /// the actual number of new contacts added.
    pub async fn bulk_add_contacts(
        &self,
        list_id: &str,
        params: &BulkAddContactsParams,
    ) -> Result<BulkAddContactsResponse, EuroMailError> {
        self.post(&format!("/v1/contact-lists/{list_id}/contacts"), params)
            .await
    }

    /// List contacts in a list with optional pagination and status filter.
    pub async fn list_contacts(
        &self,
        list_id: &str,
        params: Option<&ListContactsParams>,
    ) -> Result<PaginatedResponse<Contact>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            if let Some(page) = p.page {
                query.push(("page", page.to_string()));
            }
            if let Some(per_page) = p.per_page {
                query.push(("per_page", per_page.to_string()));
            }
            if let Some(ref status) = p.status {
                query.push(("status", status.clone()));
            }
        }
        self.get_with_query(&format!("/v1/contact-lists/{list_id}/contacts"), &query)
            .await
    }

    /// Remove a contact from a list.
    pub async fn remove_contact(
        &self,
        list_id: &str,
        contact_id: &str,
    ) -> Result<(), EuroMailError> {
        self.delete(&format!(
            "/v1/contact-lists/{list_id}/contacts/{contact_id}"
        ))
        .await
    }
}
