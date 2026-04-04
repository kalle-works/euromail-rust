use crate::client::EuroMail;
use crate::errors::EuroMailError;
use crate::types::{
    AnalyticsOverviewResponse, AnalyticsQuery, DomainAnalyticsQuery, DomainAnalyticsResponse,
    TimeseriesQuery, TimeseriesResponse,
};

impl EuroMail {
    /// Get aggregated analytics overview for the given time period.
    ///
    /// Returns totals for sent, delivered, bounced, opened, clicked, etc.
    pub async fn get_analytics_overview(
        &self,
        params: Option<&AnalyticsQuery>,
    ) -> Result<AnalyticsOverviewResponse, EuroMailError> {
        let query = analytics_query_params(params);
        self.get_with_query("/v1/analytics/overview", &query).await
    }

    /// Get daily timeseries data for selected metrics.
    pub async fn get_analytics_timeseries(
        &self,
        params: Option<&TimeseriesQuery>,
    ) -> Result<TimeseriesResponse, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            if let Some(ref period) = p.period {
                query.push(("period", period.clone()));
            }
            if let Some(ref from) = p.from {
                query.push(("from", from.clone()));
            }
            if let Some(ref to) = p.to {
                query.push(("to", to.clone()));
            }
            if let Some(ref metrics) = p.metrics {
                query.push(("metrics", metrics.clone()));
            }
        }
        self.get_with_query("/v1/analytics/timeseries", &query)
            .await
    }

    /// Get analytics broken down by sending domain.
    pub async fn get_analytics_domains(
        &self,
        params: Option<&DomainAnalyticsQuery>,
    ) -> Result<DomainAnalyticsResponse, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            if let Some(ref period) = p.period {
                query.push(("period", period.clone()));
            }
            if let Some(ref from) = p.from {
                query.push(("from", from.clone()));
            }
            if let Some(ref to) = p.to {
                query.push(("to", to.clone()));
            }
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
        }
        self.get_with_query("/v1/analytics/domains", &query).await
    }

    /// Export analytics data as a CSV string.
    pub async fn export_analytics_csv(
        &self,
        params: Option<&AnalyticsQuery>,
    ) -> Result<String, EuroMailError> {
        let query = analytics_query_params(params);
        self.get_raw("/v1/analytics/export", &query).await
    }
}

fn analytics_query_params(params: Option<&AnalyticsQuery>) -> Vec<(&str, String)> {
    let mut query: Vec<(&str, String)> = Vec::new();
    if let Some(p) = params {
        if let Some(ref period) = p.period {
            query.push(("period", period.clone()));
        }
        if let Some(ref from) = p.from {
            query.push(("from", from.clone()));
        }
        if let Some(ref to) = p.to {
            query.push(("to", to.clone()));
        }
    }
    query
}
