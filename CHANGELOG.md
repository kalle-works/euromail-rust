# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
