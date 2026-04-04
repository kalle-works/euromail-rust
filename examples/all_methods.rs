//! EuroMail Rust SDK — comprehensive example exercising every method.
//!
//! Usage:
//!     EUROMAIL_API_KEY=em_live_... cargo run --example all_methods

use euromail::*;

#[tokio::main]
async fn main() -> Result<(), EuroMailError> {
    let client =
        EuroMail::new(std::env::var("EUROMAIL_API_KEY").expect("EUROMAIL_API_KEY required"));

    // ---- Account ----
    let account = client.get_account().await?;
    println!("Account: {} ({})", account.name, account.plan);

    // ---- API Keys ----
    let api_key = client
        .create_api_key(&CreateApiKeyParams {
            name: "test-key".into(),
            scopes: Some(vec!["emails:send".into()]),
        })
        .await?;
    println!(
        "Created API key: {}... (id: {})",
        api_key.key_prefix, api_key.id
    );

    let keys = client.list_api_keys().await?;
    println!("API keys: {}", keys.len());

    client.delete_api_key(&api_key.id).await?;
    println!("Deleted API key");

    // ---- Domains ----
    let domain = client.add_domain("test-sdk-example.com").await?;
    println!("Added domain: {} (id: {})", domain.domain, domain.id);

    let domain_detail = client.get_domain(&domain.id).await?;
    println!("Domain DKIM selector: {}", domain_detail.dkim_selector);

    match client.verify_domain(&domain.id).await {
        Ok(v) => {
            if let Some(spf) = v.checks.get("spf") {
                println!("Domain SPF verified: {}", spf.verified);
            }
        }
        Err(e) => println!("Domain verify: {e}"),
    }

    let domains = client
        .list_domains(Some(&ListParams {
            page: Some(1),
            per_page: Some(10),
        }))
        .await?;
    println!("Domains: {} total", domains.data.len());

    // Tracking domain
    match client
        .set_tracking_domain(&domain.id, "track.test-sdk-example.com")
        .await
    {
        Ok(tracking) => {
            println!("Tracking domain CNAME target: {}", tracking.cname_target);
            match client.verify_tracking_domain(&domain.id).await {
                Ok(v) => println!("Tracking verified: {}", v.tracking_check.verified),
                Err(e) => println!("Tracking verify: {e}"),
            }
            let _ = client.remove_tracking_domain(&domain.id).await;
            println!("Removed tracking domain");
        }
        Err(e) => println!("Tracking domain: {e}"),
    }

    client.delete_domain(&domain.id).await?;
    println!("Deleted domain");

    // ---- Templates ----
    let template_alias = format!(
        "test-welcome-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let template = client
        .create_template(&CreateTemplateParams {
            alias: template_alias,
            name: "Test Welcome".into(),
            subject: "Welcome {{ name }}!".into(),
            html_body: Some("<p>Hello {{ name }}</p>".into()),
            text_body: None,
        })
        .await?;
    println!("Created template: {} (id: {})", template.alias, template.id);

    let tmpl = client.get_template(&template.id).await?;
    println!("Template subject: {}", tmpl.subject);

    let updated = client
        .update_template(
            &template.id,
            &UpdateTemplateParams {
                name: Some("Updated Welcome".into()),
                subject: Some(template.subject.clone()),
                html_body: Some("<p>Updated {{ name }}</p>".into()),
                ..Default::default()
            },
        )
        .await?;
    println!("Updated template name: {}", updated.name);

    let templates = client
        .list_templates(Some(&ListParams {
            page: Some(1),
            per_page: Some(10),
        }))
        .await?;
    println!("Templates: {}", templates.data.len());

    client.delete_template(&template.id).await?;
    println!("Deleted template");

    // ---- Emails ----
    let from_domain = account.email.split('@').last().unwrap_or("example.com");
    let sent = client
        .send_email(&SendEmailParams {
            from: format!("test@{from_domain}"),
            to: account.email.clone().into(),
            subject: Some("SDK test".into()),
            text_body: Some("Hello from the Rust SDK example!".into()),
            ..Default::default()
        })
        .await?;
    println!("Sent email: {} (status: {})", sent.id, sent.status);

    let email = client.get_email(&sent.id).await?;
    println!("Email to: {}", email.email.to_address);

    let emails = client
        .list_emails(
            Some(&ListParams {
                page: Some(1),
                per_page: Some(5),
            }),
            None,
        )
        .await?;
    println!("Emails: {}", emails.data.len());

    // ---- Email Validation ----
    let validation = client.validate_email("test@example.com").await?;
    println!(
        "Validation: valid={}, deliverable={}",
        validation.valid, validation.deliverable
    );

    // ---- Webhooks ----
    let webhook = client
        .create_webhook(&CreateWebhookParams {
            url: "https://httpbin.org/post".into(),
            events: vec!["delivered".into(), "bounced".into()],
        })
        .await?;
    println!("Created webhook: {}", webhook.id);

    let wh = client.get_webhook(&webhook.id).await?;
    println!("Webhook events: {}", wh.events.join(", "));

    let updated_wh = client
        .update_webhook(
            &webhook.id,
            &UpdateWebhookParams {
                url: "https://httpbin.org/post".into(),
                events: vec!["delivered".into(), "bounced".into(), "opened".into()],
                is_active: true,
            },
        )
        .await?;
    println!("Updated webhook events: {}", updated_wh.events.join(", "));

    let webhooks = client
        .list_webhooks(Some(&ListParams {
            page: Some(1),
            per_page: Some(10),
        }))
        .await?;
    println!("Webhooks: {}", webhooks.data.len());

    match client.test_webhook(&webhook.id).await {
        Ok(test) => println!("Webhook test: {}", test.message),
        Err(e) => println!("Webhook test: {e}"),
    }

    client.delete_webhook(&webhook.id).await?;
    println!("Deleted webhook");

    // ---- Suppressions ----
    let suppression = client
        .add_suppression("blocked@example.com", Some("manual"))
        .await?;
    println!("Added suppression: {}", suppression.email_address);

    let suppressions = client
        .list_suppressions(Some(&ListParams {
            page: Some(1),
            per_page: Some(10),
        }))
        .await?;
    println!("Suppressions: {}", suppressions.data.len());

    client.delete_suppression("blocked@example.com").await?;
    println!("Deleted suppression");

    // ---- Contact Lists ----
    let list = client
        .create_contact_list(&CreateContactListParams {
            name: "SDK Test List".into(),
            description: None,
            double_opt_in: None,
        })
        .await?;
    println!("Created list: {} (id: {})", list.name, list.id);

    let contact = client
        .add_contact(
            &list.id,
            &AddContactParams {
                email: "user@example.com".into(),
                metadata: None,
            },
        )
        .await?;
    println!("Added contact: {}", contact.email);

    let bulk = client
        .bulk_add_contacts(
            &list.id,
            &BulkAddContactsParams {
                contacts: vec![
                    AddContactParams {
                        email: "a@example.com".into(),
                        metadata: None,
                    },
                    AddContactParams {
                        email: "b@example.com".into(),
                        metadata: None,
                    },
                ],
            },
        )
        .await?;
    println!("Bulk added: {}/{}", bulk.inserted, bulk.total_requested);

    let contacts = client
        .list_contacts(
            &list.id,
            Some(&ListContactsParams {
                page: Some(1),
                per_page: Some(10),
                status: None,
            }),
        )
        .await?;
    println!("Contacts: {}", contacts.data.len());

    client.remove_contact(&list.id, "user@example.com").await?;
    println!("Removed contact");

    let lists = client.list_contact_lists().await?;
    println!("Contact lists: {}", lists.len());

    client.delete_contact_list(&list.id).await?;
    println!("Deleted contact list");

    // ---- Analytics ----
    let overview = client
        .get_analytics_overview(Some(&AnalyticsQuery {
            period: Some("30d".into()),
            from: None,
            to: None,
        }))
        .await?;
    println!("Analytics overview: {:?}", overview.data);

    let timeseries = client
        .get_analytics_timeseries(Some(&TimeseriesQuery {
            period: Some("7d".into()),
            from: None,
            to: None,
            metrics: None,
        }))
        .await?;
    println!("Timeseries points: {}", timeseries.data.len());

    let domain_stats = client
        .get_analytics_domains(Some(&DomainAnalyticsQuery {
            period: Some("30d".into()),
            from: None,
            to: None,
            limit: Some(5),
        }))
        .await?;
    println!("Domain analytics: {} domains", domain_stats.data.len());

    let csv = client
        .export_analytics_csv(Some(&AnalyticsQuery {
            period: Some("7d".into()),
            from: None,
            to: None,
        }))
        .await?;
    println!("CSV export: {} bytes", csv.len());

    // ---- Operations ----
    let ops = client
        .list_operations(Some(&ListParams {
            page: Some(1),
            per_page: Some(5),
        }))
        .await?;
    println!("Operations: {}", ops.data.len());

    // ---- Audit Logs ----
    let logs = client
        .list_audit_logs(Some(&ListParams {
            page: Some(1),
            per_page: Some(5),
        }))
        .await?;
    println!("Audit logs: {}", logs.data.len());

    // ---- Dead Letters ----
    let dead_letters = client.list_dead_letters(Some(5)).await?;
    println!("Dead letters: {}", dead_letters.len());

    // ---- Inbound ----
    let inbound = client
        .list_inbound_emails(Some(&ListParams {
            page: Some(1),
            per_page: Some(5),
        }))
        .await?;
    println!("Inbound emails: {}", inbound.data.len());

    let routes = client
        .list_inbound_routes(Some(&ListParams {
            page: Some(1),
            per_page: Some(5),
        }))
        .await?;
    println!("Inbound routes: {}", routes.data.len());

    // ---- Billing ----
    let plans = client.list_plans().await?;
    println!(
        "Plans: {}",
        plans
            .iter()
            .map(|p| p.plan.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let sub = client.get_subscription().await?;
    println!("Subscription: {} ({})", sub.plan, sub.subscription_status);

    // ---- GDPR ----
    match client.gdpr_export_email("test@example.com").await {
        Ok(export) => println!("GDPR export: {}", export.data.email_address),
        Err(e) => println!("GDPR export: {e}"),
    }

    println!("\nAll methods exercised successfully!");
    Ok(())
}
