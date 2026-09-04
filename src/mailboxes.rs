use serde::{Deserialize, Serialize};

use crate::client::{DataEnvelope, EuroMail};
use crate::errors::{ApiErrorBody, EuroMailError};
use crate::types::ListParams;

/// An agent mailbox — a persistent inbound email address bound to an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMailbox {
    pub id: String,
    pub account_id: String,
    pub local_part: String,
    pub domain: String,
    pub address: String,
    pub display_name: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub webhook_filters: Option<serde_json::Value>,
    #[serde(default)]
    pub auto_responder_rules: serde_json::Value,
    #[serde(default)]
    pub auto_responder_enabled: bool,
}

/// A message delivered to an agent mailbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxMessage {
    pub id: String,
    pub mailbox_id: String,
    pub account_id: String,
    pub message_id: Option<String>,
    pub mail_from: String,
    pub from_header: Option<String>,
    pub reply_to: Option<String>,
    pub subject: Option<String>,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub size_bytes: i32,
    pub thread_id: Option<String>,
    pub labels: Vec<String>,
    pub read_at: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub in_reply_to: Option<String>,
    #[serde(default)]
    pub references_header: Option<String>,
    #[serde(default)]
    pub attachments_stored: bool,
    #[serde(default)]
    pub attachments_metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub classification: Option<String>,
    #[serde(default)]
    pub classification_confidence: Option<f32>,
    #[serde(default)]
    pub classified_at: Option<String>,
    #[serde(default)]
    pub leased_until: Option<String>,
    #[serde(default)]
    pub lease_token: Option<String>,
}

/// A mailbox message returned by [`EuroMail::wait_for_next_message`], together
/// with the lease token required to ack or nack it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeasedMessage {
    pub data: MailboxMessage,
    pub lease_token: String,
    pub lease_expires_at: String,
}

/// Result of replying to a mailbox message via [`EuroMail::reply_to_message`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxReplyResult {
    pub id: String,
    pub status: String,
    pub message_id: String,
    pub to: String,
    pub subject: String,
}

/// A downloadable attachment on a mailbox message.
///
/// Normally each item carries a pre-signed download `url` (valid ~1 hour) plus
/// `content_type`, `size`, and `expires_in_seconds`. If the attachment bytes
/// were never persisted to object storage, the server instead returns the raw
/// stored metadata, which may omit `url`/`expires_in_seconds` — hence every
/// field is optional and defaults so both response shapes deserialize cleanly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxAttachmentUrl {
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub expires_in_seconds: Option<u64>,
}

/// A contact derived from the messages in a mailbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxContact {
    pub email: String,
    pub display_name: Option<String>,
    pub message_count: i64,
    pub last_seen: String,
}

/// Aggregate message statistics for a mailbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxAnalytics {
    pub total_messages: i64,
    pub unread_messages: i64,
    pub total_threads: i64,
    pub messages_today: i64,
    pub messages_this_week: i64,
}

/// Auto-responder configuration for a mailbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoResponderConfig {
    pub auto_responder_enabled: bool,
    pub auto_responder_rules: serde_json::Value,
}

/// Parameters for creating an agent mailbox.
#[derive(Debug, Default, Clone, Serialize)]
pub struct CreateMailboxParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_part: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_id: Option<String>,
}

/// Parameters for listing messages in a mailbox.
#[derive(Debug, Default, Clone, Serialize)]
pub struct ListMessagesParams {
    /// `"all"` (default), `"unread"`, or `"read"`.
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Parameters for replying to a mailbox message.
///
/// At least one of `text_body` or `html_body` must be set; the server rejects
/// the request with `400` if both are absent.
#[derive(Debug, Default, Clone, Serialize)]
pub struct ReplyToMessageParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_body: Option<String>,
}

/// Parameters for updating a mailbox's auto-responder.
///
/// `rules` is an opaque JSON array of rule objects, roughly
/// `{"match": ..., "action": {"reply_text"?: string, "reply_html"?: string}}`.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateAutoResponderParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct LeaseAck<'a> {
    lease_token: &'a str,
}

#[derive(Serialize)]
struct LabelsBody<'a> {
    labels: &'a [String],
}

#[derive(Deserialize)]
struct LabelsData {
    labels: Vec<String>,
}

impl EuroMail {
    /// Create a new agent mailbox.
    ///
    /// If `local_part` and `domain_id` are omitted, the server generates a
    /// random address on the account's default inbound domain.
    pub async fn create_mailbox(
        &self,
        params: &CreateMailboxParams,
    ) -> Result<AgentMailbox, EuroMailError> {
        self.post("/v1/agent-mailboxes", params).await
    }

    /// List agent mailboxes on the account.
    ///
    /// The `page` and `per_page` fields of [`ListParams`] are translated to the
    /// server's `offset`/`limit` pagination.
    pub async fn list_mailboxes(
        &self,
        params: Option<&ListParams>,
    ) -> Result<Vec<AgentMailbox>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            let per_page = p.per_page;
            if let Some(limit) = per_page {
                query.push(("limit", limit.to_string()));
            }
            if let (Some(page), Some(limit)) = (p.page, per_page) {
                let offset = (page.max(1) - 1) * limit;
                if offset > 0 {
                    query.push(("offset", offset.to_string()));
                }
            }
        }
        let envelope: DataEnvelope<Vec<AgentMailbox>> =
            self.get_with_query("/v1/agent-mailboxes", &query).await?;
        Ok(envelope.data)
    }

    /// Retrieve a single agent mailbox by ID.
    pub async fn get_mailbox(&self, id: &str) -> Result<AgentMailbox, EuroMailError> {
        self.get(&format!("/v1/agent-mailboxes/{id}")).await
    }

    /// Delete an agent mailbox and all its messages.
    pub async fn delete_mailbox(&self, id: &str) -> Result<(), EuroMailError> {
        self.delete(&format!("/v1/agent-mailboxes/{id}")).await
    }

    /// List messages delivered to a mailbox.
    pub async fn list_mailbox_messages(
        &self,
        mailbox_id: &str,
        params: Option<&ListMessagesParams>,
    ) -> Result<Vec<MailboxMessage>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(p) = params {
            if let Some(status) = &p.status {
                query.push(("status", status.clone()));
            }
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(offset) = p.offset {
                query.push(("offset", offset.to_string()));
            }
        }
        let envelope: DataEnvelope<Vec<MailboxMessage>> = self
            .get_with_query(
                &format!("/v1/agent-mailboxes/{mailbox_id}/messages"),
                &query,
            )
            .await?;
        Ok(envelope.data)
    }

    /// Long-poll for the next undelivered message on a mailbox.
    ///
    /// On success, returns a [`LeasedMessage`] containing the message along
    /// with a `lease_token` that must be passed to [`Self::ack_message`] or
    /// [`Self::nack_message`] within the lease window. Returns `Ok(None)` when
    /// the server responds with HTTP 408 (no message became available within
    /// `timeout_secs`).
    ///
    /// `timeout_secs` defaults to the server's default when `None`.
    pub async fn wait_for_next_message(
        &self,
        mailbox_id: &str,
        timeout_secs: Option<u64>,
    ) -> Result<Option<LeasedMessage>, EuroMailError> {
        let url = format!(
            "{}/v1/agent-mailboxes/{mailbox_id}/messages/next",
            self.base_url
        );
        let mut req = self.http.get(&url);
        if let Some(t) = timeout_secs {
            req = req.query(&[("timeout", t.to_string())]);
        }

        // Allow the HTTP request to outlive the default 30s client timeout
        // when the caller requests a longer poll.
        if let Some(t) = timeout_secs {
            if t >= 25 {
                req = req.timeout(std::time::Duration::from_secs(t + 10));
            }
        }

        let resp = req.send().await?;
        let status = resp.status().as_u16();

        if status == 408 {
            return Ok(None);
        }

        if status >= 400 {
            let retry_after = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());

            let body: ApiErrorBody = resp.json().await.unwrap_or_default();
            let (code, message, error_type) = body.resolve();

            // Same classify-by-code/type-before-status rule as EuroMail's
            // request path (see client.rs): the API returns validation
            // failures as both HTTP 400 and HTTP 422 depending on the
            // endpoint.
            let is_validation =
                error_type.as_deref() == Some("validation_error") || code == "VALIDATION_ERROR";

            return Err(if is_validation {
                EuroMailError::Validation { code, message }
            } else {
                match status {
                    401 => EuroMailError::Authentication(message),
                    404 => EuroMailError::NotFound(message),
                    429 => EuroMailError::RateLimit {
                        retry_after,
                        message,
                    },
                    _ => EuroMailError::Api {
                        status,
                        code,
                        message,
                    },
                }
            });
        }

        let leased: LeasedMessage = resp.json().await?;
        Ok(Some(leased))
    }

    /// Permanently delete a message from a mailbox.
    pub async fn delete_mailbox_message(
        &self,
        mailbox_id: &str,
        message_id: &str,
    ) -> Result<(), EuroMailError> {
        self.delete(&format!(
            "/v1/agent-mailboxes/{mailbox_id}/messages/{message_id}"
        ))
        .await
    }

    /// Acknowledge a leased message. The message will be marked as read and
    /// will not be redelivered.
    pub async fn ack_message(
        &self,
        mailbox_id: &str,
        message_id: &str,
        lease_token: &str,
    ) -> Result<(), EuroMailError> {
        let resp = self
            .http
            .post(format!(
                "{}/v1/agent-mailboxes/{mailbox_id}/messages/{message_id}/ack",
                self.base_url
            ))
            .json(&LeaseAck { lease_token })
            .send()
            .await?;
        check_status_empty(resp).await
    }

    /// Negative-acknowledge a leased message. The lease is released
    /// immediately and the message becomes available for redelivery.
    pub async fn nack_message(
        &self,
        mailbox_id: &str,
        message_id: &str,
        lease_token: &str,
    ) -> Result<(), EuroMailError> {
        let resp = self
            .http
            .post(format!(
                "{}/v1/agent-mailboxes/{mailbox_id}/messages/{message_id}/nack",
                self.base_url
            ))
            .json(&LeaseAck { lease_token })
            .send()
            .await?;
        check_status_empty(resp).await
    }

    /// Send a reply to a message in a mailbox.
    ///
    /// At least one of `text_body`/`html_body` must be set on `params`; the
    /// server rejects the request otherwise.
    pub async fn reply_to_message(
        &self,
        mailbox_id: &str,
        message_id: &str,
        params: ReplyToMessageParams,
    ) -> Result<MailboxReplyResult, EuroMailError> {
        self.post(
            &format!("/v1/agent-mailboxes/{mailbox_id}/messages/{message_id}/reply"),
            &params,
        )
        .await
    }

    /// List threads in a mailbox, one row per thread (its latest message).
    pub async fn list_mailbox_threads(
        &self,
        mailbox_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<MailboxMessage>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(offset) = offset {
            query.push(("offset", offset.to_string()));
        }
        let envelope: DataEnvelope<Vec<MailboxMessage>> = self
            .get_with_query(&format!("/v1/agent-mailboxes/{mailbox_id}/threads"), &query)
            .await?;
        Ok(envelope.data)
    }

    /// Retrieve a full thread, returned chronologically ascending.
    pub async fn get_mailbox_thread(
        &self,
        mailbox_id: &str,
        thread_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<MailboxMessage>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(offset) = offset {
            query.push(("offset", offset.to_string()));
        }
        let envelope: DataEnvelope<Vec<MailboxMessage>> = self
            .get_with_query(
                &format!("/v1/agent-mailboxes/{mailbox_id}/threads/{thread_id}"),
                &query,
            )
            .await?;
        Ok(envelope.data)
    }

    /// Full-text search messages in a mailbox.
    ///
    /// `query` must be 1–500 characters; the server rejects an empty or
    /// over-long query.
    pub async fn search_mailbox_messages(
        &self,
        mailbox_id: &str,
        query: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<MailboxMessage>, EuroMailError> {
        let mut q: Vec<(&str, String)> = vec![("q", query.to_string())];
        if let Some(limit) = limit {
            q.push(("limit", limit.to_string()));
        }
        if let Some(offset) = offset {
            q.push(("offset", offset.to_string()));
        }
        let envelope: DataEnvelope<Vec<MailboxMessage>> = self
            .get_with_query(
                &format!("/v1/agent-mailboxes/{mailbox_id}/messages/search"),
                &q,
            )
            .await?;
        Ok(envelope.data)
    }

    /// Replace the labels on a message (full replace, not a merge).
    ///
    /// Each label must be 1–64 characters of alphanumerics, dashes, or
    /// underscores; at most 50 labels are allowed.
    pub async fn update_message_labels(
        &self,
        mailbox_id: &str,
        message_id: &str,
        labels: &[String],
    ) -> Result<Vec<String>, EuroMailError> {
        let data: LabelsData = self
            .put(
                &format!("/v1/agent-mailboxes/{mailbox_id}/messages/{message_id}/labels"),
                &LabelsBody { labels },
            )
            .await?;
        Ok(data.labels)
    }

    /// Get downloadable URLs for a message's attachments.
    ///
    /// See [`MailboxAttachmentUrl`] for the fallback shape returned when the
    /// attachment bytes were never persisted to object storage.
    pub async fn get_message_attachment_urls(
        &self,
        mailbox_id: &str,
        message_id: &str,
    ) -> Result<Vec<MailboxAttachmentUrl>, EuroMailError> {
        self.get(&format!(
            "/v1/agent-mailboxes/{mailbox_id}/messages/{message_id}/attachments"
        ))
        .await
    }

    /// List contacts derived from a mailbox's messages.
    pub async fn list_mailbox_contacts(
        &self,
        mailbox_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<MailboxContact>, EuroMailError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(offset) = offset {
            query.push(("offset", offset.to_string()));
        }
        let envelope: DataEnvelope<Vec<MailboxContact>> = self
            .get_with_query(
                &format!("/v1/agent-mailboxes/{mailbox_id}/contacts"),
                &query,
            )
            .await?;
        Ok(envelope.data)
    }

    /// Retrieve aggregate message statistics for a mailbox.
    pub async fn get_mailbox_analytics(
        &self,
        mailbox_id: &str,
    ) -> Result<MailboxAnalytics, EuroMailError> {
        self.get(&format!("/v1/agent-mailboxes/{mailbox_id}/analytics"))
            .await
    }

    /// Update a mailbox's auto-responder configuration.
    pub async fn update_auto_responder(
        &self,
        mailbox_id: &str,
        params: UpdateAutoResponderParams,
    ) -> Result<AutoResponderConfig, EuroMailError> {
        self.patch(
            &format!("/v1/agent-mailboxes/{mailbox_id}/auto-responder"),
            &params,
        )
        .await
    }
}

async fn check_status_empty(resp: reqwest::Response) -> Result<(), EuroMailError> {
    let status = resp.status().as_u16();
    if status < 400 {
        return Ok(());
    }

    let retry_after = resp
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());

    let body: ApiErrorBody = resp.json().await.unwrap_or_default();
    let (code, message, error_type) = body.resolve();

    let is_validation =
        error_type.as_deref() == Some("validation_error") || code == "VALIDATION_ERROR";

    Err(if is_validation {
        EuroMailError::Validation { code, message }
    } else {
        match status {
            401 => EuroMailError::Authentication(message),
            404 => EuroMailError::NotFound(message),
            429 => EuroMailError::RateLimit {
                retry_after,
                message,
            },
            _ => EuroMailError::Api {
                status,
                code,
                message,
            },
        }
    })
}
