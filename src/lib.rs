//! # EuroMail
//!
//! Official Rust SDK for the [EuroMail](https://euromail.dev) transactional email service.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use euromail::{EuroMail, SendEmailParams};
//!
//! # async fn run() -> Result<(), euromail::EuroMailError> {
//! let client = EuroMail::new("em_live_your_api_key");
//!
//! let email = client.send_email(
//!     &SendEmailParams::new("you@yourdomain.com", "user@example.com")
//!         .subject("Hello from EuroMail")
//!         .text_body("Welcome!"),
//! ).await?;
//!
//! println!("Sent email: {}", email.id);
//! # Ok(())
//! # }
//! ```
//!
//! ## Configuration
//!
//! Create a client with your API key. The base URL defaults to `https://api.euromail.dev`
//! and can be overridden via the `EUROMAIL_API_URL` environment variable or
//! [`EuroMail::with_base_url`].
//!
//! ## Features
//!
//! - **Emails** — send, batch send, cancel, and track delivery status
//! - **Templates** — create and manage reusable email templates
//! - **Domains** — register and verify sending domains (SPF, DKIM, DMARC)
//! - **Webhooks** — subscribe to delivery events (sent, bounced, opened, etc.)
//! - **Contact lists** — manage subscriber lists with bulk operations
//! - **Suppressions** — maintain bounce/complaint suppression lists
//! - **Analytics** — query delivery metrics, timeseries, and per-domain breakdowns
//! - **Inbound** — receive and route incoming emails
//! - **Agent mailboxes** — persistent mailboxes for AI agents with lease/ack/nack delivery
//! - **API keys** — create scoped keys with fine-grained permissions
//! - **GDPR** — export and erase personal data
//! - **Audit logs** — review account activity
//! - **Dead letters** — inspect and retry failed deliveries
//! - **Insights** — trigger AI-generated operational reports
//!
//! ## Verifying webhook signatures
//!
//! ```rust,no_run
//! use euromail::verify_webhook_signature;
//!
//! # fn handle_request(body: &[u8], signature_header: &str, secret: &str) -> Result<(), Box<dyn std::error::Error>> {
//! verify_webhook_signature(body, signature_header, secret)?;
//! // Safe to trust: this request came from EuroMail.
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod errors;
pub mod mailboxes;
pub mod types;
pub mod webhook_signature;

mod account;
mod analytics;
mod api_keys;
mod audit_logs;
mod billing;
mod contact_lists;
mod dead_letters;
mod domains;
mod emails;
mod gdpr;
mod inbound;
mod inbound_routes;
mod insights;
mod newsletters;
mod operations;
mod signup_forms;
mod sub_accounts;
mod suppressions;
mod templates;
mod validate;
mod webhooks;

pub use client::EuroMail;
pub use errors::EuroMailError;
pub use mailboxes::{
    AgentMailbox, AutoResponderConfig, CreateMailboxParams, LeasedMessage, ListMessagesParams,
    MailboxAnalytics, MailboxAttachmentUrl, MailboxContact, MailboxMessage, MailboxReplyResult,
    ReplyToMessageParams, UpdateAutoResponderParams,
};
pub use types::*;
pub use webhook_signature::{
    DEFAULT_TOLERANCE_SECONDS, WebhookSignatureError, verify_webhook_signature,
    verify_webhook_signature_at,
};
