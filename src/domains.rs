use serde::Serialize;

use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{
    Domain, DomainVerificationResult, ListParams, PaginatedResponse, TrackingDomainResponse,
    TrackingDomainVerification,
};

#[derive(Serialize)]
struct AddDomainBody {
    domain: String,
}

impl EuroMail {
    /// Register a new sending domain.
    ///
    /// Returns the domain with DNS records that must be added before sending.
    pub async fn add_domain(&self, domain: &str) -> Result<Domain, EuroMailError> {
        let body = AddDomainBody {
            domain: domain.to_string(),
        };
        self.post("/v1/domains", &body).await
    }

    /// Get a domain by ID.
    pub async fn get_domain(&self, domain_id: &str) -> Result<Domain, EuroMailError> {
        self.get(&format!("/v1/domains/{domain_id}")).await
    }

    /// Trigger DNS verification for a domain.
    ///
    /// Checks SPF, DKIM, DMARC, and return-path records. Returns the current
    /// verification status for each record type.
    pub async fn verify_domain(
        &self,
        domain_id: &str,
    ) -> Result<DomainVerificationResult, EuroMailError> {
        self.post(
            &format!("/v1/domains/{domain_id}/verify"),
            &serde_json::json!({}),
        )
        .await
    }

    /// Delete a domain and stop sending from it.
    pub async fn delete_domain(&self, domain_id: &str) -> Result<(), EuroMailError> {
        self.delete(&format!("/v1/domains/{domain_id}")).await
    }

    /// Set a vanity tracking domain for click/open tracking links.
    pub async fn set_tracking_domain(
        &self,
        domain_id: &str,
        tracking_domain: &str,
    ) -> Result<TrackingDomainResponse, EuroMailError> {
        self.put_direct(
            &format!("/v1/domains/{domain_id}/tracking-domain"),
            &serde_json::json!({ "tracking_domain": tracking_domain }),
        )
        .await
    }

    /// Verify the CNAME record for a vanity tracking domain.
    pub async fn verify_tracking_domain(
        &self,
        domain_id: &str,
    ) -> Result<TrackingDomainVerification, EuroMailError> {
        self.post_direct(
            &format!("/v1/domains/{domain_id}/verify-tracking"),
            &serde_json::json!({}),
        )
        .await
    }

    /// Remove the vanity tracking domain.
    pub async fn remove_tracking_domain(&self, domain_id: &str) -> Result<(), EuroMailError> {
        self.delete(&format!("/v1/domains/{domain_id}/tracking-domain"))
            .await
    }

    /// List all domains with optional pagination.
    pub async fn list_domains(
        &self,
        params: Option<&ListParams>,
    ) -> Result<PaginatedResponse<Domain>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            if let Some(page) = p.page {
                query.push(("page", page.to_string()));
            }
            if let Some(per_page) = p.per_page {
                query.push(("per_page", per_page.to_string()));
            }
        }
        self.get_with_query("/v1/domains", &query).await
    }
}
