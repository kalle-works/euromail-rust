use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize `null` JSON values as `Default::default()` instead of failing.
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

// ---- Account ----

/// Your EuroMail account details and quota usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    /// Current plan name (e.g. `"free"`, `"pro"`, `"enterprise"`).
    pub plan: String,
    /// Maximum emails allowed per billing cycle.
    pub monthly_quota: i64,
    /// Emails already sent in the current billing cycle.
    pub emails_sent_this_month: i64,
    /// ISO 8601 timestamp when the quota counter resets.
    pub quota_reset_at: String,
    pub created_at: String,
}

// ---- Email Types ----

/// Parameters for sending a single email.
///
/// At minimum, `from` and `to` are required. Provide either `html_body`/`text_body`
/// directly, or reference a template via `template_alias` with `template_data`.
///
/// # Example
///
/// ```rust
/// use euromail::SendEmailParams;
///
/// let params = SendEmailParams {
///     from: "hello@yourdomain.com".into(),
///     to: "user@example.com".into(),
///     subject: Some("Welcome".into()),
///     html_body: Some("<h1>Hello!</h1>".into()),
///     ..Default::default()
/// };
/// ```
/// Accepts a single recipient or a list of recipients.
///
/// Serializes as a string when there's exactly one recipient, or as an array
/// when there are multiple.
#[derive(Debug, Clone)]
pub enum Recipient {
    /// Single recipient: `"user@example.com"`
    One(String),
    /// Multiple recipients: `["a@example.com", "b@example.com"]`
    Many(Vec<String>),
}

impl Default for Recipient {
    fn default() -> Self {
        Recipient::One(String::new())
    }
}

impl From<String> for Recipient {
    fn from(s: String) -> Self {
        Recipient::One(s)
    }
}

impl From<&str> for Recipient {
    fn from(s: &str) -> Self {
        Recipient::One(s.to_string())
    }
}

impl From<Vec<String>> for Recipient {
    fn from(v: Vec<String>) -> Self {
        if v.len() == 1 {
            Recipient::One(v.into_iter().next().unwrap())
        } else {
            Recipient::Many(v)
        }
    }
}

impl Serialize for Recipient {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Recipient::One(s) => serializer.serialize_str(s),
            Recipient::Many(v) => v.serialize(serializer),
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct SendEmailParams {
    /// Sender address (must belong to a verified domain).
    pub from: String,
    /// Single recipient or list of recipients.
    pub to: Recipient,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_body: Option<String>,
    /// Use a stored template by alias instead of inline body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_alias: Option<String>,
    /// Variables to interpolate into the template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_data: Option<serde_json::Value>,
    /// Custom SMTP headers to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<serde_json::Value>,
    /// Tags for categorizing and filtering emails in analytics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Arbitrary key-value metadata attached to the email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// File attachments (base64-encoded content).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
    /// Prevents duplicate sends when retrying — the API de-duplicates on this key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

/// A file attachment for an email.
#[derive(Debug, Clone, Serialize)]
pub struct Attachment {
    pub filename: String,
    /// Base64-encoded file content.
    pub content: String,
    /// MIME type (e.g. `"application/pdf"`).
    pub content_type: String,
}

/// Response after successfully queuing an email for delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailResponse {
    pub id: String,
    /// RFC 5322 Message-ID assigned by EuroMail.
    pub message_id: String,
    pub status: String,
    pub to: String,
    pub created_at: String,
}

/// Parameters for sending multiple emails in a single request (up to 100).
#[derive(Debug, Clone, Serialize)]
pub struct SendBatchParams {
    pub emails: Vec<SendEmailParams>,
}

/// Response from a batch send — contains both successes and per-email errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendBatchResponse {
    /// Successfully queued emails.
    pub data: Vec<SendEmailResponse>,
    /// Emails that failed validation, identified by their index in the request.
    #[serde(default)]
    pub errors: Vec<BatchError>,
}

/// A single error within a batch send response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// Zero-based index of the failed email in the request array.
    pub index: i32,
    pub error: String,
}

/// Delivery status of an email.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EmailStatus {
    Queued,
    Processing,
    Sent,
    Delivered,
    Bounced,
    Failed,
    Rejected,
}

/// Full email record with delivery metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    pub account_id: String,
    pub domain_id: Option<String>,
    pub message_id: String,
    pub from_address: String,
    pub to_address: String,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub reply_to: Option<String>,
    pub subject: String,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
    pub template_id: Option<String>,
    pub template_data: Option<serde_json::Value>,
    pub status: EmailStatus,
    /// Number of delivery attempts made so far.
    pub attempts: i32,
    pub max_attempts: i32,
    /// Last error message if delivery failed.
    pub error_message: Option<String>,
    /// Raw SMTP response from the receiving server.
    pub smtp_response: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub metadata: HashMap<String, String>,
    pub headers: Option<serde_json::Value>,
    pub attachments: Option<serde_json::Value>,
    pub idempotency_key: Option<String>,
    pub operation_id: Option<String>,
    pub next_retry_at: Option<String>,
    pub scheduled_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub sent_at: Option<String>,
}

/// A delivery lifecycle event (sent, delivered, bounced, opened, clicked, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailEvent {
    pub id: String,
    pub email_id: String,
    pub account_id: String,
    /// Event type: `"sent"`, `"delivered"`, `"bounced"`, `"opened"`, `"clicked"`, etc.
    pub event_type: String,
    /// Set for bounce events: `"hard"` or `"soft"`.
    pub bounce_type: Option<String>,
    pub bounce_category: Option<String>,
    pub created_at: String,
}

/// An email together with its delivery events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDetail {
    pub email: Email,
    pub events: Vec<EmailEvent>,
}

// ---- Template Types ----

/// A reusable email template stored in your account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub account_id: String,
    /// Unique alias used to reference this template when sending.
    pub alias: String,
    pub name: String,
    /// Subject line (supports `{{variable}}` interpolation).
    pub subject: String,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Parameters for creating a new template.
#[derive(Debug, Clone, Serialize)]
pub struct CreateTemplateParams {
    /// Unique alias for referencing this template in send calls.
    pub alias: String,
    pub name: String,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_body: Option<String>,
}

/// Parameters for updating an existing template. Only provided fields are changed.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateTemplateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_body: Option<String>,
}

// ---- Domain Types ----

/// A sending domain registered with your account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: String,
    pub account_id: String,
    pub domain: String,
    /// DKIM selector (used in the `s=` DNS record).
    pub dkim_selector: String,
    /// DKIM public key (RSA).
    #[serde(default)]
    pub dkim_public_key: Option<String>,
    pub spf_verified: bool,
    pub dkim_verified: bool,
    pub dmarc_verified: bool,
    pub return_path_verified: bool,
    pub mx_verified: bool,
    pub mx_verified_at: Option<String>,
    pub inbound_enabled: bool,
    /// DNS records keyed by purpose (e.g. `"spf"`, `"dkim"`, `"dmarc"`, `"return_path"`, `"bounce_spf"`).
    pub dns_records: HashMap<String, DnsRecord>,
    pub verified_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A DNS record required for domain verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    /// Record type: `"TXT"`, `"CNAME"`, or `"MX"`.
    #[serde(rename = "type")]
    pub type_: String,
    /// DNS hostname (e.g. `"euromail._domainkey.yourdomain.com"`).
    pub host: String,
    /// Record value to set.
    pub value: String,
    /// MX priority (only present for MX records).
    pub priority: Option<u32>,
}

/// Result of a single DNS verification check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub verified: bool,
    pub detail: String,
}

/// Result of a domain verification check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainVerificationResult {
    pub domain: Domain,
    pub checks: HashMap<String, VerificationCheck>,
}

// ---- Webhook Types ----

/// A webhook endpoint subscribed to delivery events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: String,
    pub account_id: String,
    /// HTTPS URL that receives POST requests with event payloads.
    pub url: String,
    /// Event types this webhook listens for (e.g. `["sent", "bounced", "opened"]`).
    pub events: Vec<String>,
    pub is_active: bool,
    /// Number of consecutive delivery failures.
    pub failure_count: Option<i32>,
    pub last_success_at: Option<String>,
    pub last_failure_at: Option<String>,
    pub last_failure_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Parameters for creating a new webhook.
#[derive(Debug, Clone, Serialize)]
pub struct CreateWebhookParams {
    /// HTTPS URL to receive webhook POST requests.
    pub url: String,
    /// Event types to subscribe to (e.g. `["sent", "bounced", "opened"]`).
    pub events: Vec<String>,
}

/// Parameters for updating a webhook.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateWebhookParams {
    pub url: String,
    pub events: Vec<String>,
    pub is_active: bool,
}

/// Response from testing a webhook endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookTestResponse {
    pub message: String,
    /// The test payload that was sent to the webhook URL.
    pub payload: serde_json::Value,
}

// ---- Suppression Types ----

/// A suppressed email address that will not receive emails.
///
/// Suppressions are automatically created for hard bounces and spam complaints.
/// You can also add them manually.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suppression {
    pub id: String,
    pub account_id: String,
    pub email_address: String,
    /// Reason for suppression: `"hard_bounce"`, `"complaint"`, `"manual"`, `"unsubscribe"`, or `"fbl"` (ISP feedback loop).
    pub reason: String,
    /// The email that triggered this suppression, if any.
    pub source_email_id: Option<String>,
    pub created_at: String,
}

// ---- Contact List Types ----

/// A named list of email contacts for batch operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactList {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub description: Option<String>,
    /// Whether new contacts require email confirmation before becoming active.
    pub double_opt_in: bool,
    #[serde(default)]
    pub contact_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Parameters for creating a new contact list.
#[derive(Debug, Clone, Serialize)]
pub struct CreateContactListParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_opt_in: Option<bool>,
}

/// Parameters for updating a contact list.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateContactListParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub double_opt_in: bool,
}

/// A contact within a contact list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub list_id: String,
    pub email: String,
    pub metadata: Option<serde_json::Value>,
    /// Contact status: `"active"`, `"unsubscribed"`, `"bounced"`, or `"pending"`.
    pub status: String,
    pub created_at: String,
}

/// Parameters for adding a single contact to a list.
#[derive(Debug, Clone, Serialize)]
pub struct AddContactParams {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Parameters for adding multiple contacts in a single request.
#[derive(Debug, Clone, Serialize)]
pub struct BulkAddContactsParams {
    pub contacts: Vec<AddContactParams>,
}

/// Response from a bulk contact add operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkAddContactsResponse {
    /// Number of contacts actually inserted (duplicates are skipped).
    pub inserted: i64,
    pub total_requested: i64,
}

/// Query parameters for listing contacts with optional filters.
#[derive(Debug, Clone)]
pub struct ListContactsParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    /// Filter by contact status (e.g. `"active"`, `"unsubscribed"`).
    pub status: Option<String>,
}

// ---- Analytics Types ----

/// Query parameters for analytics endpoints.
#[derive(Debug, Clone)]
pub struct AnalyticsQuery {
    /// Preset period: `"24h"`, `"7d"`, `"30d"`, or `"90d"`.
    pub period: Option<String>,
    /// Custom start date (ISO 8601).
    pub from: Option<String>,
    /// Custom end date (ISO 8601).
    pub to: Option<String>,
}

/// The time period covered by an analytics response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsPeriod {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub period: Option<String>,
}

/// Aggregated analytics overview (totals for the period).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsOverviewResponse {
    pub data: serde_json::Value,
    pub period: AnalyticsPeriod,
}

/// Query parameters for timeseries analytics.
#[derive(Debug, Clone)]
pub struct TimeseriesQuery {
    pub period: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    /// Comma-separated metric names: `"sent,delivered,bounced,opens,clicks"`.
    pub metrics: Option<String>,
}

/// A single data point in a timeseries response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesPoint {
    /// Date in `YYYY-MM-DD` format.
    pub date: String,
    pub sent: Option<i64>,
    pub delivered: Option<i64>,
    pub bounced: Option<i64>,
    pub opens: Option<i64>,
    pub clicks: Option<i64>,
}

/// Timeseries analytics response with daily data points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesResponse {
    pub data: Vec<TimeseriesPoint>,
    pub period: AnalyticsPeriod,
}

/// Query parameters for per-domain analytics.
#[derive(Debug, Clone)]
pub struct DomainAnalyticsQuery {
    pub period: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    /// Maximum number of domains to return.
    pub limit: Option<i64>,
}

/// Deserialize a float that may be encoded as a JSON string (e.g. `"0.0"`).
fn deserialize_f64_or_string<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum FloatOrString {
        Float(f64),
        Str(String),
    }
    match FloatOrString::deserialize(deserializer)? {
        FloatOrString::Float(f) => Ok(f),
        FloatOrString::Str(s) => s.parse().map_err(serde::de::Error::custom),
    }
}

/// Analytics breakdown for a single sending domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAnalytics {
    pub domain: String,
    pub sent: i64,
    pub delivered: i64,
    pub bounced: i64,
    #[serde(deserialize_with = "deserialize_f64_or_string")]
    pub open_rate: f64,
    #[serde(deserialize_with = "deserialize_f64_or_string")]
    pub click_rate: f64,
}

/// Per-domain analytics response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAnalyticsResponse {
    pub data: Vec<DomainAnalytics>,
    pub period: AnalyticsPeriod,
}

// ---- Audit Log Types ----

/// An audit log entry recording an account action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub account_id: String,
    /// Action performed (e.g. `"domain.created"`, `"api_key.deleted"`).
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: String,
}

// ---- Dead Letter Types ----

/// An email that permanently failed delivery after all retry attempts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetter {
    pub stream_id: String,
    pub original_stream: String,
    pub email_id: String,
    pub account_id: String,
    pub failure_reason: String,
    pub attempt_count: i32,
    pub last_error: String,
    pub failed_at: String,
    pub payload: serde_json::Value,
}

// ---- Inbound Email Types ----

/// An email received on one of your verified domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundEmail {
    pub id: String,
    pub account_id: String,
    pub domain_id: String,
    pub from_address: String,
    pub to_addresses: Vec<String>,
    pub subject: String,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    /// Size of the raw email in bytes.
    pub raw_size: i64,
    pub created_at: String,
}

// ---- Inbound Route Types ----

/// A routing rule that forwards inbound emails to a webhook URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundRoute {
    pub id: String,
    pub account_id: String,
    pub domain_id: String,
    /// Address pattern to match (e.g. `"support@"`, `"*@"`).
    pub pattern: String,
    /// Match strategy: `"prefix"`, `"exact"`, or `"catchall"`.
    pub match_type: String,
    /// Lower numbers are evaluated first.
    pub priority: i32,
    /// URL that receives matched inbound emails via POST.
    pub webhook_url: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Parameters for creating an inbound routing rule.
#[derive(Debug, Clone, Serialize)]
pub struct CreateInboundRouteParams {
    /// ID of the domain this route applies to.
    pub domain_id: String,
    pub pattern: String,
    pub match_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
}

/// Parameters for updating an inbound route. Only provided fields are changed.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateInboundRouteParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

// ---- API Key Types ----

/// Parameters for creating a new API key.
#[derive(Debug, Clone, Serialize)]
pub struct CreateApiKeyParams {
    /// Human-readable name for the key.
    pub name: String,
    /// Permission scopes (e.g. `["email:send", "domains:read"]`). Omit for full access.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

/// Response from creating an API key — includes the full key (shown only once).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    pub id: String,
    pub name: String,
    /// The full API key. Store this securely — it cannot be retrieved again.
    pub key: String,
    /// First characters of the key for identification (e.g. `"em_live_abc"`).
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub created_at: String,
}

/// An API key (without the secret portion).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub last_used_at: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

// ---- GDPR Types ----

/// Response from a GDPR data export request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprExportResponse {
    pub data: GdprExportData,
    pub exported_at: String,
}

/// All personal data associated with an email address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprExportData {
    pub email_address: String,
    pub emails: Vec<serde_json::Value>,
    pub events: Vec<serde_json::Value>,
    pub suppressions: Vec<serde_json::Value>,
    pub unsubscribe_events: Vec<serde_json::Value>,
    pub inbound_emails: Vec<serde_json::Value>,
}

/// Response from a GDPR erasure request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprEraseResponse {
    pub data: GdprEraseData,
}

/// Result of erasing all data for an email address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprEraseData {
    pub email_address: String,
    /// Total number of database rows deleted across all tables.
    pub rows_deleted: i64,
    pub message: String,
}

// ---- Sub-Account Types ----

/// A sub-account managed by a parent account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAccount {
    pub id: String,
    pub name: String,
    pub email: String,
    pub plan: String,
    pub monthly_quota: i64,
    pub emails_sent_this_month: i64,
    pub parent_account_id: String,
    pub is_active: bool,
    pub created_at: String,
}

/// Parameters for creating a new sub-account.
#[derive(Debug, Clone, Serialize)]
pub struct CreateSubAccountParams {
    pub name: String,
    pub email: String,
    pub password: String,
    pub monthly_quota: i32,
}

/// Parameters for updating a sub-account. Only provided fields are changed.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateSubAccountParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_quota: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

// ---- Broadcast Types ----

/// Parameters for broadcasting an email to a contact list.
#[derive(Debug, Clone, Serialize, Default)]
pub struct BroadcastParams {
    pub contact_list_id: String,
    pub from_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_at: Option<String>,
}

/// Response from a broadcast send.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastResponse {
    pub operation_id: String,
    pub total_recipients: i32,
    pub message: String,
}

// ---- Tracking Domain Types ----

/// Response from setting a tracking domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingDomainResponse {
    pub data: Domain,
    pub cname_target: String,
}

/// Response from verifying a tracking domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingDomainVerification {
    pub data: Domain,
    pub tracking_check: TrackingCheck,
}

/// Result of a tracking domain CNAME check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingCheck {
    pub verified: bool,
    pub detail: String,
}

// ---- Newsletter Types ----

/// A newsletter draft or sent newsletter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Newsletter {
    pub id: String,
    pub account_id: String,
    pub list_id: Option<String>,
    pub subject: String,
    pub from_address: String,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
    pub template_id: Option<String>,
    pub template_data: Option<serde_json::Value>,
    pub reply_to: Option<String>,
    pub status: String,
    pub operation_id: Option<String>,
    pub scheduled_at: Option<String>,
    pub sent_at: Option<String>,
    pub total_recipients: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

/// Parameters for creating a newsletter.
#[derive(Debug, Clone, Serialize)]
pub struct CreateNewsletterParams {
    pub list_id: String,
    pub subject: String,
    pub from_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
}

/// Parameters for updating a newsletter. Only provided fields are changed.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateNewsletterParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
}

/// Response from sending a newsletter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsletterSendResponse {
    pub operation_id: String,
    pub total_recipients: i32,
    pub message: String,
}

// ---- Signup Form Types ----

/// A signup form for collecting newsletter subscribers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupForm {
    pub id: String,
    pub account_id: String,
    pub list_id: String,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub success_message: Option<String>,
    pub redirect_url: Option<String>,
    pub custom_fields: serde_json::Value,
    pub theme: serde_json::Value,
    pub is_active: bool,
    pub form_url: String,
    pub embed_code: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Parameters for creating a signup form.
#[derive(Debug, Clone, Serialize)]
pub struct CreateSignupFormParams {
    pub list_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<serde_json::Value>,
}

/// Parameters for updating a signup form. Only provided fields are changed.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateSignupFormParams {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<serde_json::Value>,
}

// ---- Email Validation Types ----

/// Response from validating an email address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailValidation {
    pub email: String,
    pub valid: bool,
    pub deliverable: String,
    pub is_disposable: bool,
    pub is_role: bool,
    pub is_free: bool,
    pub mx_found: bool,
    pub reason: Option<String>,
}

// ---- Operation Types ----

/// An async operation (broadcast, newsletter send, bulk import).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    pub account_id: String,
    pub operation_type: String,
    pub status: String,
    pub total_items: i32,
    pub completed_items: i32,
    pub failed_items: i32,
    pub error_summary: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub expires_at: String,
}

// ---- Billing Types ----

/// A billing plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingPlan {
    pub plan: String,
    pub monthly_quota: i64,
    pub max_domains: i32,
    pub max_templates: i32,
    pub max_webhooks: i32,
    pub max_contact_lists: i32,
    pub max_sub_accounts: i32,
    pub tracking_enabled: bool,
    pub price_cents: i32,
    pub stripe_price_id: Option<String>,
}

/// Subscription limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionLimits {
    pub max_domains: i32,
    pub max_templates: i32,
    pub max_webhooks: i32,
    pub tracking_enabled: bool,
    pub price_cents: i32,
}

/// Current billing subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub plan: String,
    pub subscription_status: String,
    pub stripe_subscription_id: Option<String>,
    pub billing_email: Option<String>,
    pub trial_ends_at: Option<String>,
    pub monthly_quota: i64,
    pub emails_sent_this_month: i64,
    pub limits: SubscriptionLimits,
}

/// Parameters for creating a Stripe checkout session.
#[derive(Debug, Clone, Serialize)]
pub struct CheckoutParams {
    pub plan: String,
    pub success_url: String,
    pub cancel_url: String,
}

/// Response containing a Stripe checkout URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutResponse {
    pub checkout_url: String,
}

/// Parameters for creating a Stripe billing portal session.
#[derive(Debug, Clone, Serialize)]
pub struct PortalParams {
    pub return_url: String,
}

/// Response containing a Stripe billing portal URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalResponse {
    pub portal_url: String,
}

// ---- Link Click Stats ----

/// Per-link click statistics for a sent email.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkClickStat {
    /// The tracked URL.
    pub url: String,
    /// Total number of clicks on this link.
    pub clicks: i64,
    /// Number of unique recipients who clicked this link.
    pub unique_clicks: i64,
}

// ---- Insight Types ----

/// A single finding produced by the AI insights engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightFinding {
    /// Severity level: `"info"`, `"warn"`, or `"critical"`.
    pub severity: String,
    /// Functional area: `"deliverability"`, `"reputation"`, `"performance"`, or `"security"`.
    pub area: String,
    /// What was observed in the operational data.
    pub observation: String,
    /// Recommended action to address the finding.
    pub recommendation: String,
}

/// An AI-generated operational insights report for an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightReport {
    pub id: String,
    pub account_id: Option<String>,
    pub generated_at: String,
    pub period_start: String,
    pub period_end: String,
    /// Claude model used to generate this report.
    pub model: String,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    /// One- or two-sentence headline summary.
    pub summary: String,
    /// Structured findings with severity, area, observation, and recommendation.
    pub findings: Vec<InsightFinding>,
    /// Full markdown report body as returned by Claude.
    pub raw_markdown: Option<String>,
    pub acknowledged_at: Option<String>,
}

// ---- Pagination ----

/// Pagination parameters for list endpoints.
#[derive(Debug, Clone)]
pub struct ListParams {
    /// Page number (1-based).
    pub page: Option<i64>,
    /// Items per page (default varies by endpoint, max 100).
    pub per_page: Option<i64>,
}

/// A paginated response wrapping a list of items.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize",
    deserialize = "T: serde::de::DeserializeOwned"
))]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: Pagination,
}

/// Pagination metadata returned with list responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}
