# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2026-07-07

### Added

- Agent mailbox parity methods on `EuroMail`:
  - `reply_to_message` — send a reply to a mailbox message.
  - `list_mailbox_threads` / `get_mailbox_thread` — browse threads and read a
    full thread chronologically.
  - `search_mailbox_messages` — full-text search within a mailbox.
  - `update_message_labels` — replace the labels on a message.
  - `get_message_attachment_urls` — fetch pre-signed attachment download URLs.
  - `list_mailbox_contacts` — list contacts derived from a mailbox's messages.
  - `get_mailbox_analytics` — aggregate message statistics for a mailbox.
  - `update_auto_responder` — configure a mailbox's auto-responder.
- New public types: `MailboxReplyResult`, `MailboxAttachmentUrl`,
  `MailboxContact`, `MailboxAnalytics`, `AutoResponderConfig`,
  `ReplyToMessageParams`, `UpdateAutoResponderParams`.
- Additional fields on `AgentMailbox` (`webhook_filters`,
  `auto_responder_rules`, `auto_responder_enabled`) and `MailboxMessage`
  (`in_reply_to`, `references_header`, `attachments_stored`,
  `attachments_metadata`, `classification`, `classification_confidence`,
  `classified_at`, `leased_until`, `lease_token`).

## [0.6.0] - 2026-07-07

### Added

- `SendEmailParams` builder: `.transactional(bool)` and `.stream(impl Into<String>)`,
  matching the current `/v1/emails` API (added 2026-05).
- `BroadcastParams`: `tracking` and `transactional` fields.

### Changed

- **BREAKING:** Removed `SendEmailParams::suppress_list_management_header` (field
  and builder method). The server dropped this field from the public API on
  2026-05-03 in favor of `transactional`, so it had become a silent no-op —
  callers relying on it got no error, but the field was never read server-side.
  Since single-send `transactional` defaults to `true` server-side and this
  crate had no way to set it to `false`, every email sent through this SDK was
  silently treated as transactional (no `List-Unsubscribe` header), even
  intentional marketing/newsletter sends. Replace
  `.suppress_list_management_header(true)` with `.transactional(true)` (the
  default) and `.suppress_list_management_header(false)` with
  `.transactional(false)`.

## [0.5.0] - 2026-04-23

### Added

- Fluent builder API on `SendEmailParams`: `new(from, to)` plus chainable
  setters for every optional field (`subject`, `html_body`, `text_body`,
  `template_alias`, `template_data`, `headers`, `tags`, `tag`, `metadata`,
  `metadatum`, `attachments`, `attach`, `reply_to`, `cc`, `bcc`,
  `idempotency_key`, `send_at`, `tracking`, `suppress_list_management_header`)
- Fluent builder API on `ConfigureWelcomeEmailParams`: `new`, `enable`,
  `disable`, plus chainable `subject`, `html_body`, `text_body`, `template_id`,
  `from_address`, `delay_seconds`

### Changed

- **BREAKING:** `SendEmailParams` and `ConfigureWelcomeEmailParams` are now
  marked `#[non_exhaustive]`. Downstream crates can no longer construct these
  types with a struct literal — use the builder constructors (`::new(...)`)
  and chained setters instead. This makes future field additions non-breaking.

## [0.4.0] - 2026-04-23

### Added

- `SendEmailParams` now carries `send_at` (RFC 3339 delayed delivery), `tracking`
  (per-email open/click tracking override), and `suppress_list_management_header`
  (strip `List-Unsubscribe` headers on transactional mail)
- `SendEmailResponse` now exposes `sandbox` and `scheduled_at` returned by the API,
  and is marked `#[non_exhaustive]` so future fields are additive
- Contact list welcome-email configuration: `get_welcome_email` and
  `configure_welcome_email`, backed by new `WelcomeEmailConfig` (marked
  `#[non_exhaustive]`) and `ConfigureWelcomeEmailParams` types
- `ContactList` gains `welcome_email_enabled`, `welcome_email_subject`,
  `welcome_email_html_body`, `welcome_email_text_body`, `welcome_email_template_id`,
  `welcome_email_from_address`, `welcome_email_delay_seconds` so the value returned
  by `configure_welcome_email` surfaces the saved config (all default when the
  plain list/get endpoints omit them)
- `MAX_WELCOME_DELAY_SECONDS` constant (7 days) for client-side bounds checking

## [0.3.0] - 2026-04-13

### Added

- Native agent mailbox support: `create_mailbox`, `list_mailboxes`, `get_mailbox`,
  `delete_mailbox`, `list_mailbox_messages`, `delete_mailbox_message`
- `wait_for_next_message` long-poll method that returns `Ok(None)` on HTTP 408
  when no message arrives within the timeout window
- `ack_message` / `nack_message` for the at-least-once lease/ack/nack delivery model
- New types: `AgentMailbox`, `MailboxMessage`, `LeasedMessage`, `CreateMailboxParams`,
  `ListMessagesParams`

### Changed

- README "Agent Mailboxes" section now demonstrates native SDK usage instead of
  raw `reqwest` calls

## [0.2.0] - 2026-04-13

### Added

- `EuroMail::from_env()` constructor that reads `EUROMAIL_API_KEY` from environment
- `get_email_links` method for retrieving tracked links from sent emails
- `generate_insights` method for triggering AI-generated operational reports
- New response types for links and insights endpoints

### Fixed

- Repository URL in Cargo.toml now points to the correct `euromail-rust` repo

## [0.1.0] - 2026-03-15

### Added

- Initial Rust SDK for euromail transactional email API
- Email sending, batch send, cancel, and delivery status tracking
- Template management (create, update, delete, list)
- Domain registration and verification (SPF, DKIM, DMARC)
- Webhook subscriptions for delivery events
- Contact list management with bulk operations
- Suppression list management (bounces, complaints)
- Analytics queries (delivery metrics, timeseries, per-domain breakdowns)
- Inbound email receiving and routing
- API key management with scoped permissions
- GDPR data export and erasure
- Audit log access
- Dead letter inspection and retry
- Comprehensive error handling with typed error variants
- `EUROMAIL_API_URL` environment variable support for custom base URLs
