use euromail::{EuroMail, Recipient, SendBatchParams, SendEmailParams};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_send_email() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/emails"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "email-456",
                "message_id": "<msg-456@euromail.dev>",
                "status": "queued",
                "to": "recipient@example.com",
                "created_at": "2026-03-07T12:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let params = SendEmailParams {
        from: "sender@example.com".to_string(),
        to: Recipient::One("recipient@example.com".to_string()),
        subject: Some("Hello".to_string()),
        html_body: Some("<h1>Hi</h1>".to_string()),
        text_body: None,
        cc: None,
        bcc: None,
        reply_to: None,
        template_alias: None,
        template_data: None,
        headers: None,
        tags: None,
        metadata: None,
        attachments: None,
        idempotency_key: None,
    };

    let response = client.send_email(&params).await.unwrap();
    assert_eq!(response.id, "email-456");
    assert_eq!(response.status, "queued");
    assert_eq!(response.to, "recipient@example.com");
}

#[tokio::test]
async fn test_send_batch() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/emails/batch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "email-1",
                    "message_id": "<msg-1@euromail.dev>",
                    "status": "queued",
                    "to": "a@example.com",
                    "created_at": "2026-03-07T12:00:00Z"
                },
                {
                    "id": "email-2",
                    "message_id": "<msg-2@euromail.dev>",
                    "status": "queued",
                    "to": "b@example.com",
                    "created_at": "2026-03-07T12:00:00Z"
                }
            ],
            "errors": []
        })))
        .mount(&mock_server)
        .await;

    let params = SendBatchParams {
        emails: vec![
            SendEmailParams {
                from: "sender@example.com".to_string(),
                to: Recipient::One("a@example.com".to_string()),
                subject: Some("Hello A".to_string()),
                html_body: Some("<p>Hi A</p>".to_string()),
                text_body: None,
                cc: None,
                bcc: None,
                reply_to: None,
                template_alias: None,
                template_data: None,
                headers: None,
                tags: None,
                metadata: None,
                attachments: None,
                idempotency_key: None,
            },
            SendEmailParams {
                from: "sender@example.com".to_string(),
                to: Recipient::One("b@example.com".to_string()),
                subject: Some("Hello B".to_string()),
                html_body: Some("<p>Hi B</p>".to_string()),
                text_body: None,
                cc: None,
                bcc: None,
                reply_to: None,
                template_alias: None,
                template_data: None,
                headers: None,
                tags: None,
                metadata: None,
                attachments: None,
                idempotency_key: None,
            },
        ],
    };

    let response = client.send_batch(&params).await.unwrap();
    assert_eq!(response.data.len(), 2);
    assert_eq!(response.data[0].id, "email-1");
    assert_eq!(response.data[1].id, "email-2");
    assert!(response.errors.is_empty());
}

#[tokio::test]
async fn test_get_email() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/emails/email-789"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "email": {
                    "id": "email-789",
                    "account_id": "acc-123",
                    "domain_id": null,
                    "message_id": "<msg-789@euromail.dev>",
                    "from_address": "sender@example.com",
                    "to_address": "recipient@example.com",
                    "cc": null,
                    "bcc": null,
                    "reply_to": null,
                    "subject": "Test Email",
                    "html_body": "<h1>Hi</h1>",
                    "text_body": null,
                    "template_id": null,
                    "status": "delivered",
                    "attempts": 1,
                    "max_attempts": 3,
                    "error_message": null,
                    "smtp_response": null,
                    "tags": ["newsletter"],
                    "metadata": {},
                    "created_at": "2026-03-07T12:00:00Z",
                    "updated_at": "2026-03-07T12:01:00Z",
                    "sent_at": "2026-03-07T12:00:30Z"
                },
                "events": [
                    {
                        "id": "evt-1",
                        "email_id": "email-789",
                        "account_id": "acc-123",
                        "event_type": "delivered",
                        "bounce_type": null,
                        "bounce_category": null,
                        "created_at": "2026-03-07T12:01:00Z"
                    }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    let detail = client.get_email("email-789").await.unwrap();
    assert_eq!(detail.email.id, "email-789");
    assert_eq!(detail.email.status, euromail::EmailStatus::Delivered);
    assert_eq!(detail.events.len(), 1);
    assert_eq!(detail.events[0].event_type, "delivered");
}

#[tokio::test]
async fn test_list_emails() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/emails"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "email-1",
                    "account_id": "acc-123",
                    "domain_id": null,
                    "message_id": "<msg-1@euromail.dev>",
                    "from_address": "sender@example.com",
                    "to_address": "a@example.com",
                    "cc": null,
                    "bcc": null,
                    "reply_to": null,
                    "subject": "Test",
                    "html_body": null,
                    "text_body": "Hello",
                    "template_id": null,
                    "status": "sent",
                    "attempts": 1,
                    "max_attempts": 3,
                    "error_message": null,
                    "smtp_response": null,
                    "tags": [],
                    "metadata": {},
                    "created_at": "2026-03-07T12:00:00Z",
                    "updated_at": "2026-03-07T12:00:30Z",
                    "sent_at": "2026-03-07T12:00:30Z"
                }
            ],
            "pagination": {
                "page": 1,
                "per_page": 25,
                "total": 1,
                "total_pages": 1
            }
        })))
        .mount(&mock_server)
        .await;

    let result = client.list_emails(None, None).await.unwrap();
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].id, "email-1");
    assert_eq!(result.pagination.total, 1);
}

#[tokio::test]
async fn test_cancel_email() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/emails/email-scheduled-1/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "email-scheduled-1",
                "message_id": "<msg-sched@euromail.dev>",
                "status": "failed",
                "to": "recipient@example.com",
                "created_at": "2026-03-16T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let response = client.cancel_email("email-scheduled-1").await.unwrap();
    assert_eq!(response.id, "email-scheduled-1");
    assert_eq!(response.status, "failed");
}

#[tokio::test]
async fn test_send_email_with_default() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/emails"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "email-default",
                "message_id": "<msg-default@euromail.dev>",
                "status": "queued",
                "to": "recipient@example.com",
                "created_at": "2026-03-16T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let params = SendEmailParams {
        from: "sender@example.com".to_string(),
        to: Recipient::One("recipient@example.com".to_string()),
        subject: Some("Hello".to_string()),
        html_body: Some("<p>Hi</p>".to_string()),
        ..Default::default()
    };

    let response = client.send_email(&params).await.unwrap();
    assert_eq!(response.id, "email-default");
}
