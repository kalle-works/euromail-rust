# euromail

Official Rust SDK for the [EuroMail](https://euromail.dev) transactional email service.

[![Crates.io](https://img.shields.io/crates/v/euromail.svg)](https://crates.io/crates/euromail)
[![docs.rs](https://docs.rs/euromail/badge.svg)](https://docs.rs/euromail)

## Installation

```toml
[dependencies]
euromail = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use euromail::{EuroMail, SendEmailParams};

#[tokio::main]
async fn main() -> Result<(), euromail::EuroMailError> {
    let client = EuroMail::new("em_live_your_api_key_here");

    let response = client.send_email(&SendEmailParams {
        from: "sender@yourdomain.com".into(),
        to: "recipient@example.com".into(),
        subject: Some("Hello from EuroMail".into()),
        html_body: Some("<h1>Welcome!</h1>".into()),
        ..Default::default()
    }).await?;

    println!("Email queued: {}", response.id);
    Ok(())
}
```

## Configuration

```rust
let client = EuroMail::new("em_live_...");
```

## Sending Emails

### Direct send

```rust
use euromail::SendEmailParams;
use std::collections::HashMap;

let response = client.send_email(&SendEmailParams {
    from: "noreply@yourdomain.com".into(),
    to: "user@example.com".into(),
    subject: Some("Order Confirmation".into()),
    html_body: Some("<h1>Thanks for your order!</h1>".into()),
    text_body: Some("Thanks for your order!".into()),
    reply_to: Some("support@yourdomain.com".into()),
    tags: Some(vec!["order".into(), "confirmation".into()]),
    metadata: Some(HashMap::from([("order_id".into(), "12345".into())])),
    ..Default::default()
}).await?;
```

### Send with template

```rust
use serde_json::json;

let response = client.send_email(&SendEmailParams {
    from: "noreply@yourdomain.com".into(),
    to: "user@example.com".into(),
    template_alias: Some("welcome-email".into()),
    template_data: Some(json!({
        "name": "John",
        "activation_url": "https://example.com/activate/abc123"
    })),
    ..Default::default()
}).await?;
```

### Send with attachments

```rust
use euromail::{SendEmailParams, Attachment};

let response = client.send_email(&SendEmailParams {
    from: "noreply@yourdomain.com".into(),
    to: "user@example.com".into(),
    subject: Some("Your Invoice".into()),
    html_body: Some("<p>Please find your invoice attached.</p>".into()),
    attachments: Some(vec![Attachment {
        filename: "invoice.pdf".into(),
        content: base64_encoded_content,
        content_type: "application/pdf".into(),
    }]),
    ..Default::default()
}).await?;
```

### Batch send

```rust
use euromail::SendBatchParams;

let batch = client.send_batch(&SendBatchParams {
    emails: vec![
        SendEmailParams {
            from: "noreply@yourdomain.com".into(),
            to: "user1@example.com".into(),
            subject: Some("Hello User 1".into()),
            text_body: Some("Welcome!".into()),
            ..Default::default()
        },
        SendEmailParams {
            from: "noreply@yourdomain.com".into(),
            to: "user2@example.com".into(),
            subject: Some("Hello User 2".into()),
            text_body: Some("Welcome!".into()),
            ..Default::default()
        },
    ],
}).await?;

println!("Sent: {}, Errors: {}", batch.data.len(), batch.errors.len());
```

### Retrieve and list emails

```rust
let email = client.get_email("email-uuid").await?;

let emails = client.list_emails(Some(&euromail::ListParams {
    page: Some(1),
    per_page: Some(50),
})).await?;
```

## Domains

```rust
// Register a sending domain
let domain = client.add_domain("mail.yourdomain.com").await?;
println!("Configure DNS records: {:?}", domain.dns_records);

// Trigger verification
let verification = client.verify_domain(&domain.id).await?;
if verification.fully_verified {
    println!("Domain verified!");
}

// List all domains
let domains = client.list_domains(None).await?;

// Remove a domain
client.delete_domain(&domain.id).await?;
```

## Templates

```rust
use euromail::{CreateTemplateParams, UpdateTemplateParams};

let template = client.create_template(&CreateTemplateParams {
    alias: "welcome".into(),
    name: "Welcome Email".into(),
    subject: "Welcome, {{name}}!".into(),
    html_body: Some("<h1>Welcome, {{name}}!</h1>".into()),
    text_body: None,
}).await?;

// Update
client.update_template(&template.id, &UpdateTemplateParams {
    subject: Some("Welcome to {{company}}, {{name}}!".into()),
    ..Default::default()
}).await?;

// List and delete
let templates = client.list_templates(None).await?;
client.delete_template(&template.id).await?;
```

## Webhooks

```rust
use euromail::{CreateWebhookParams, UpdateWebhookParams};

let webhook = client.create_webhook(&CreateWebhookParams {
    url: "https://yourdomain.com/webhooks/euromail".into(),
    events: vec!["delivered".into(), "bounced".into(), "complained".into()],
}).await?;

// Update
client.update_webhook(&webhook.id, &UpdateWebhookParams {
    url: "https://yourdomain.com/webhooks/v2".into(),
    events: vec!["delivered".into(), "bounced".into()],
    is_active: true,
}).await?;

// Send test event
let test = client.test_webhook(&webhook.id).await?;

// List and delete
let webhooks = client.list_webhooks(None).await?;
client.delete_webhook(&webhook.id).await?;
```

Supported events: `sent`, `delivered`, `bounced`, `opened`, `clicked`, `complained`, `email.inbound`

## Suppressions

```rust
client.add_suppression("bounced@example.com", Some("hard_bounce")).await?;

let suppressions = client.list_suppressions(None).await?;

client.delete_suppression("bounced@example.com").await?;
```

## Contact Lists

```rust
use euromail::{CreateContactListParams, AddContactParams, BulkAddContactsParams};

let list = client.create_contact_list(&CreateContactListParams {
    name: "Newsletter".into(),
    description: Some("Monthly product updates".into()),
    double_opt_in: Some(true),
}).await?;

// Add a single contact
let contact = client.add_contact(&list.id, &AddContactParams {
    email: "user@example.com".into(),
    metadata: None,
}).await?;

// Bulk add
let result = client.bulk_add_contacts(&list.id, &BulkAddContactsParams {
    contacts: vec![
        AddContactParams { email: "a@example.com".into(), metadata: None },
        AddContactParams { email: "b@example.com".into(), metadata: None },
    ],
}).await?;
println!("Inserted: {} of {}", result.inserted, result.total_requested);

// List contacts
let contacts = client.list_contacts(&list.id, None).await?;

// Remove contact and delete list
client.remove_contact(&list.id, "user@example.com").await?;
client.delete_contact_list(&list.id).await?;
```

## Inbound Email

```rust
let inbound = client.list_inbound_emails(None).await?;

let email = client.get_inbound_email("inbound-uuid").await?;
println!("From: {}, Subject: {}", email.from_address, email.subject);

client.delete_inbound_email("inbound-uuid").await?;
```

## Inbound Routes

```rust
use euromail::CreateInboundRouteParams;

// Route incoming email to a webhook
let route = client.create_inbound_route(&CreateInboundRouteParams {
    domain_id: "domain-uuid".into(),
    pattern: "support@".into(),
    match_type: "prefix".into(),
    priority: Some(10),
    webhook_url: Some("https://yourdomain.com/inbound/support".into()),
}).await?;

// List and delete
let routes = client.list_inbound_routes(None).await?;
client.delete_inbound_route(&route.id).await?;
```

## Analytics

```rust
use euromail::{AnalyticsQuery, TimeseriesQuery, DomainAnalyticsQuery};

// Overview for the last 30 days
let overview = client.get_analytics_overview(Some(&AnalyticsQuery {
    period: Some("30d".into()),
    from: None,
    to: None,
})).await?;

// Time series
let timeseries = client.get_analytics_timeseries(Some(&TimeseriesQuery {
    period: Some("7d".into()),
    from: None,
    to: None,
    metrics: Some("sent,delivered,bounced".into()),
})).await?;

// Per-domain breakdown
let domains = client.get_analytics_domains(Some(&DomainAnalyticsQuery {
    period: Some("30d".into()),
    from: None,
    to: None,
    limit: Some(10),
})).await?;

// Export as CSV
let csv = client.export_analytics_csv(None).await?;
```

## Account

```rust
let account = client.get_account().await?;
println!("Plan: {}, Used: {}/{}", account.plan, account.emails_sent_this_month, account.monthly_quota);

// Export account data (GDPR)
let export = client.export_account().await?;

// Delete account permanently
client.delete_account().await?;
```

## Audit Logs

```rust
let logs = client.list_audit_logs(None).await?;
for log in &logs.data {
    println!("{}: {} on {}", log.created_at, log.action, log.resource_type);
}
```

## Dead Letters

```rust
let dead_letters = client.list_dead_letters(None).await?;

client.retry_dead_letter("dead-letter-uuid").await?;

client.delete_dead_letter("dead-letter-uuid").await?;
```

## Error Handling

All methods return `Result<T, EuroMailError>`:

```rust
use euromail::EuroMailError;

match client.send_email(&params).await {
    Ok(response) => println!("Sent: {}", response.id),
    Err(EuroMailError::Authentication(msg)) => {
        eprintln!("Invalid API key: {msg}");
    }
    Err(EuroMailError::Validation { code, message }) => {
        eprintln!("Validation error [{code}]: {message}");
    }
    Err(EuroMailError::RateLimit { retry_after, message }) => {
        eprintln!("Rate limited: {message}");
        if let Some(secs) = retry_after {
            tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
        }
    }
    Err(EuroMailError::NotFound(msg)) => {
        eprintln!("Not found: {msg}");
    }
    Err(EuroMailError::Api { status, code, message }) => {
        eprintln!("API error [{status}] {code}: {message}");
    }
    Err(EuroMailError::Http(e)) => {
        eprintln!("Network error: {e}");
    }
}
```

| Variant | HTTP Status | Description |
|---|---|---|
| `Authentication` | 401 | Invalid or missing API key |
| `Validation` | 422 | Invalid request parameters |
| `RateLimit` | 429 | Too many requests (includes `retry_after`) |
| `NotFound` | 404 | Resource does not exist |
| `Api` | 4xx/5xx | Other API errors |
| `Http` | - | Network or transport errors |

## License

MIT
