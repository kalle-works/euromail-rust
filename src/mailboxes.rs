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
}

/// A mailbox message returned by [`EuroMail::wait_for_next_message`], together
/// with the lease token required to ack or nack it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeasedMessage {
    pub data: MailboxMessage,
    pub lease_token: String,
    pub lease_expires_at: String,
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

#[derive(Serialize)]
struct LeaseAck<'a> {
    lease_token: &'a str,
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

            let body: ApiErrorBody = resp.json().await.unwrap_or(ApiErrorBody {
                code: "unknown".to_string(),
                message: "Unknown error".to_string(),
            });

            return Err(match status {
                401 => EuroMailError::Authentication(body.message),
                404 => EuroMailError::NotFound(body.message),
                422 => EuroMailError::Validation {
                    code: body.code,
                    message: body.message,
                },
                429 => EuroMailError::RateLimit {
                    retry_after,
                    message: body.message,
                },
                _ => EuroMailError::Api {
                    status,
                    code: body.code,
                    message: body.message,
                },
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

    let body: ApiErrorBody = resp.json().await.unwrap_or(ApiErrorBody {
        code: "unknown".to_string(),
        message: "Unknown error".to_string(),
    });

    Err(match status {
        401 => EuroMailError::Authentication(body.message),
        404 => EuroMailError::NotFound(body.message),
        422 => EuroMailError::Validation {
            code: body.code,
            message: body.message,
        },
        429 => EuroMailError::RateLimit {
            retry_after,
            message: body.message,
        },
        _ => EuroMailError::Api {
            status,
            code: body.code,
            message: body.message,
        },
    })
}
