use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{
    BillingPlan, CheckoutParams, CheckoutResponse, PortalParams, PortalResponse, Subscription,
};

impl EuroMail {
    /// List available billing plans.
    pub async fn list_plans(&self) -> Result<Vec<BillingPlan>, EuroMailError> {
        #[derive(serde::Deserialize)]
        struct Resp {
            data: Vec<BillingPlan>,
        }
        let resp: Resp = self.get_direct("/v1/billing/plans").await?;
        Ok(resp.data)
    }

    /// Get the current billing subscription.
    pub async fn get_subscription(&self) -> Result<Subscription, EuroMailError> {
        self.get("/v1/billing/subscription").await
    }

    /// Create a Stripe checkout session for upgrading.
    pub async fn create_checkout(
        &self,
        params: &CheckoutParams,
    ) -> Result<CheckoutResponse, EuroMailError> {
        self.post("/v1/billing/checkout", params).await
    }

    /// Create a Stripe billing portal session.
    pub async fn create_billing_portal(
        &self,
        params: &PortalParams,
    ) -> Result<PortalResponse, EuroMailError> {
        self.post("/v1/billing/portal", params).await
    }
}
