use euromail::{CreateMailboxParams, EuroMail};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_create_mailbox() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/agent-mailboxes"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "mbx_1",
                "account_id": "acc_1",
                "local_part": "agent",
                "domain": "mail.example.com",
                "address": "agent@mail.example.com",
                "display_name": "Support Agent",
                "created_at": "2026-04-13T12:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let mailbox = client
        .create_mailbox(&CreateMailboxParams {
            display_name: Some("Support Agent".to_string()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(mailbox.id, "mbx_1");
    assert_eq!(mailbox.address, "agent@mail.example.com");
    assert_eq!(mailbox.display_name.as_deref(), Some("Support Agent"));
}

#[tokio::test]
async fn test_list_mailboxes() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/agent-mailboxes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "mbx_1",
                    "account_id": "acc_1",
                    "local_part": "agent",
                    "domain": "mail.example.com",
                    "address": "agent@mail.example.com",
                    "display_name": null,
                    "created_at": "2026-04-13T12:00:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let mailboxes = client.list_mailboxes(None).await.unwrap();
    assert_eq!(mailboxes.len(), 1);
    assert_eq!(mailboxes[0].id, "mbx_1");
}

#[tokio::test]
async fn test_wait_for_next_message_success() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/agent-mailboxes/mbx_1/messages/next"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "msg_1",
                "mailbox_id": "mbx_1",
                "account_id": "acc_1",
                "message_id": "<xyz@example.com>",
                "mail_from": "user@example.com",
                "from_header": "User <user@example.com>",
                "reply_to": null,
                "subject": "Hello",
                "text_body": "Hi there",
                "html_body": null,
                "size_bytes": 42,
                "thread_id": null,
                "labels": [],
                "read_at": null,
                "created_at": "2026-04-13T12:00:00Z"
            },
            "lease_token": "lease-abc",
            "lease_expires_at": "2026-04-13T12:05:00Z"
        })))
        .mount(&mock_server)
        .await;

    let leased = client
        .wait_for_next_message("mbx_1", Some(5))
        .await
        .unwrap()
        .expect("should return Some message");
    assert_eq!(leased.data.id, "msg_1");
    assert_eq!(leased.lease_token, "lease-abc");
}

#[tokio::test]
async fn test_wait_for_next_message_408_returns_none() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/agent-mailboxes/mbx_1/messages/next"))
        .respond_with(ResponseTemplate::new(408).set_body_json(serde_json::json!({
            "code": "timeout",
            "message": "No message available"
        })))
        .mount(&mock_server)
        .await;

    let result = client
        .wait_for_next_message("mbx_1", Some(1))
        .await
        .unwrap();
    assert!(result.is_none(), "408 should map to None");
}

#[tokio::test]
async fn test_ack_message() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/agent-mailboxes/mbx_1/messages/msg_1/ack"))
        .and(body_json(serde_json::json!({ "lease_token": "lease-abc" })))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    client
        .ack_message("mbx_1", "msg_1", "lease-abc")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_nack_message() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/agent-mailboxes/mbx_1/messages/msg_1/nack"))
        .and(body_json(serde_json::json!({ "lease_token": "lease-abc" })))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    client
        .nack_message("mbx_1", "msg_1", "lease-abc")
        .await
        .unwrap();
}
