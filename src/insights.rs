use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::InsightReport;

impl EuroMail {
    /// Trigger an AI-powered operational insights report for this account.
    ///
    /// Analyses the last 7 days of operational data (bounce rates, complaint
    /// rates, throttled domains, FBL reports) and returns a structured report
    /// with up to three findings, each with severity, area, observation, and
    /// recommendation.
    ///
    /// Requires the `account:admin` scope. At most one report can be generated
    /// per account per 24-hour period.
    ///
    /// # Errors
    ///
    /// Returns [`EuroMailError::Api`] with status 503 if the Anthropic API key
    /// is not configured on the server.
    pub async fn generate_insights(&self) -> Result<InsightReport, EuroMailError> {
        self.post_direct("/v1/insights/generate", &()).await
    }
}
