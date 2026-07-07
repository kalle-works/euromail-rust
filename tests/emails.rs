use euromail::{EuroMail, SendBatchParams, SendEmailParams};
use wiremock::matchers::{body_partial_json, header, method, path};
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

    let params = SendEmailParams::new("sender@example.com", "recipient@example.com")
        .subject("Hello")
        .html_body("<h1>Hi</h1>");

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
            SendEmailParams::new("sender@example.com", "a@example.com")
                .subject("Hello A")
                .html_body("<p>Hi A</p>"),
            SendEmailParams::new("sender@example.com", "b@example.com")
                .subject("Hello B")
                .html_body("<p>Hi B</p>"),
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

    let params = SendEmailParams::new("sender@example.com", "recipient@example.com")
        .subject("Hello")
        .html_body("<p>Hi</p>");

    let response = client.send_email(&params).await.unwrap();
    assert_eq!(response.id, "email-default");
}

#[tokio::test]
async fn test_send_email_scheduled_with_tracking_override() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/emails"))
        .and(body_partial_json(serde_json::json!({
            "send_at": "2026-05-01T09:00:00Z",
            "tracking": false,
            "transactional": true,
            "stream": "transactional"
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "data": {
                "id": "email-scheduled",
                "message_id": "<sched@euromail.dev>",
                "status": "scheduled",
                "to": "recipient@example.com",
                "sandbox": false,
                "scheduled_at": "2026-05-01T09:00:00Z",
                "created_at": "2026-04-23T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let params = SendEmailParams::new("sender@example.com", "recipient@example.com")
        .subject("Reminder")
        .text_body("Hi")
        .send_at("2026-05-01T09:00:00Z")
        .tracking(false)
        .transactional(true)
        .stream("transactional");

    let response = client.send_email(&params).await.unwrap();
    assert_eq!(response.status, "scheduled");
    assert!(!response.sandbox);
    assert_eq!(
        response.scheduled_at.as_deref(),
        Some("2026-05-01T09:00:00Z")
    );
}

#[tokio::test]
async fn test_send_email_marketing_stream() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/emails"))
        .and(body_partial_json(serde_json::json!({
            "transactional": false,
            "stream": "marketing"
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "data": {
                "id": "email-marketing",
                "message_id": "<marketing@euromail.dev>",
                "status": "queued",
                "to": "recipient@example.com",
                "sandbox": false,
                "scheduled_at": null,
                "created_at": "2026-07-07T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let params = SendEmailParams::new("news@example.com", "recipient@example.com")
        .subject("This week in your inbox")
        .html_body("<p>Latest updates...</p>")
        .transactional(false)
        .stream("marketing");

    let response = client.send_email(&params).await.unwrap();
    assert_eq!(response.id, "email-marketing");
}

#[tokio::test]
async fn test_send_email_tracking_force_on() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/emails"))
        .and(body_partial_json(serde_json::json!({ "tracking": true })))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "data": {
                "id": "email-tracked",
                "message_id": "<t@euromail.dev>",
                "status": "queued",
                "to": "recipient@example.com",
                "sandbox": false,
                "created_at": "2026-04-23T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let params = SendEmailParams::new("sender@example.com", "recipient@example.com")
        .subject("Promo")
        .text_body("Body")
        .tracking(true);

    let response = client.send_email(&params).await.unwrap();
    assert_eq!(response.status, "queued");
}

#[tokio::test]
async fn test_send_email_sandbox_response() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/emails"))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "data": {
                "id": "email-sandbox",
                "message_id": "<sb@euromail.dev>",
                "status": "queued",
                "to": "recipient@example.com",
                "sandbox": true,
                "scheduled_at": null,
                "created_at": "2026-04-23T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let params = SendEmailParams::new("sender@unverified.example", "recipient@example.com")
        .subject("Hi")
        .text_body("Body");

    let response = client.send_email(&params).await.unwrap();
    assert!(response.sandbox);
    assert!(response.scheduled_at.is_none());
}
