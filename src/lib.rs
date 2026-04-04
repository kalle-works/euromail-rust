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
//! let email = client.send_email(&SendEmailParams {
//!     from: "you@yourdomain.com".into(),
//!     to: "user@example.com".into(),
//!     subject: Some("Hello from EuroMail".into()),
//!     text_body: Some("Welcome!".into()),
//!     ..Default::default()
//! }).await?;
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
//! - **API keys** — create scoped keys with fine-grained permissions
//! - **GDPR** — export and erase personal data
//! - **Audit logs** — review account activity
//! - **Dead letters** — inspect and retry failed deliveries

pub mod client;
pub mod errors;
pub mod types;

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
pub use types::*;
