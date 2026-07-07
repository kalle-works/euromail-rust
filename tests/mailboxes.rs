use euromail::{CreateMailboxParams, EuroMail, ReplyToMessageParams, UpdateAutoResponderParams};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A minimal mailbox-message JSON body for reuse in list/thread/search tests.
fn message_json(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
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
        "thread_id": "thr_1",
        "labels": [],
        "read_at": null,
        "created_at": "2026-04-13T12:00:00Z"
    })
}

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

#[tokio::test]
async fn test_reply_to_message() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/agent-mailboxes/mbx_1/messages/msg_1/reply"))
        .and(body_json(serde_json::json!({ "text_body": "Thanks!" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {
                "id": "eml_1",
                "status": "queued",
                "message_id": "<reply@example.com>",
                "to": "user@example.com",
                "subject": "Re: Hello"
            }
        })))
        .mount(&mock_server)
        .await;

    let result = client
        .reply_to_message(
            "mbx_1",
            "msg_1",
            ReplyToMessageParams {
                text_body: Some("Thanks!".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(result.id, "eml_1");
    assert_eq!(result.status, "queued");
    assert_eq!(result.to, "user@example.com");
    assert_eq!(result.subject, "Re: Hello");
}

#[tokio::test]
async fn test_list_mailbox_threads() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/agent-mailboxes/mbx_1/threads"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [message_json("msg_1")]
        })))
        .mount(&mock_server)
        .await;

    let threads = client
        .list_mailbox_threads("mbx_1", Some(10), None)
        .await
        .unwrap();
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].thread_id.as_deref(), Some("thr_1"));
}

#[tokio::test]
async fn test_get_mailbox_thread() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/agent-mailboxes/mbx_1/threads/thr_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [message_json("msg_1"), message_json("msg_2")]
        })))
        .mount(&mock_server)
        .await;

    let thread = client
        .get_mailbox_thread("mbx_1", "thr_1", None, None)
        .await
        .unwrap();
    assert_eq!(thread.len(), 2);
    assert_eq!(thread[1].id, "msg_2");
}

#[tokio::test]
async fn test_search_mailbox_messages() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/agent-mailboxes/mbx_1/messages/search"))
        .and(query_param("q", "invoice status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [message_json("msg_1")]
        })))
        .mount(&mock_server)
        .await;

    let results = client
        .search_mailbox_messages("mbx_1", "invoice status", None, None)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "msg_1");
}

#[tokio::test]
async fn test_update_message_labels() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("PUT"))
        .and(path("/v1/agent-mailboxes/mbx_1/messages/msg_1/labels"))
        .and(body_json(
            serde_json::json!({ "labels": ["urgent", "billing"] }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "labels": ["urgent", "billing"] }
        })))
        .mount(&mock_server)
        .await;

    let labels = client
        .update_message_labels(
            "mbx_1",
            "msg_1",
            &["urgent".to_string(), "billing".to_string()],
        )
        .await
        .unwrap();
    assert_eq!(labels, vec!["urgent".to_string(), "billing".to_string()]);
}

#[tokio::test]
async fn test_get_message_attachment_urls() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/agent-mailboxes/mbx_1/messages/msg_1/attachments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "filename": "invoice.pdf",
                "content_type": "application/pdf",
                "size": 12345,
                "url": "https://storage.example.com/invoice.pdf?sig=abc",
                "expires_in_seconds": 3600
            }]
        })))
        .mount(&mock_server)
        .await;

    let attachments = client
        .get_message_attachment_urls("mbx_1", "msg_1")
        .await
        .unwrap();
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].filename.as_deref(), Some("invoice.pdf"));
    assert_eq!(attachments[0].size, Some(12345));
    assert_eq!(attachments[0].expires_in_seconds, Some(3600));
}

#[tokio::test]
async fn test_get_message_attachment_urls_fallback_metadata() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    // Fallback shape: bytes never persisted, so raw stored metadata lacks
    // `url`/`expires_in_seconds` and may carry only partial fields.
    Mock::given(method("GET"))
        .and(path("/v1/agent-mailboxes/mbx_1/messages/msg_1/attachments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "filename": "note.txt",
                "content_type": "text/plain"
            }]
        })))
        .mount(&mock_server)
        .await;

    let attachments = client
        .get_message_attachment_urls("mbx_1", "msg_1")
        .await
        .unwrap();
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].filename.as_deref(), Some("note.txt"));
    assert!(attachments[0].url.is_none());
    assert!(attachments[0].expires_in_seconds.is_none());
    assert!(attachments[0].size.is_none());
}

#[tokio::test]
async fn test_list_mailbox_contacts() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/agent-mailboxes/mbx_1/contacts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "email": "user@example.com",
                "display_name": "User",
                "message_count": 7,
                "last_seen": "2026-04-13T12:00:00Z"
            }]
        })))
        .mount(&mock_server)
        .await;

    let contacts = client
        .list_mailbox_contacts("mbx_1", None, None)
        .await
        .unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].email, "user@example.com");
    assert_eq!(contacts[0].message_count, 7);
}

#[tokio::test]
async fn test_get_mailbox_analytics() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/agent-mailboxes/mbx_1/analytics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "total_messages": 100,
                "unread_messages": 5,
                "total_threads": 20,
                "messages_today": 3,
                "messages_this_week": 15
            }
        })))
        .mount(&mock_server)
        .await;

    let analytics = client.get_mailbox_analytics("mbx_1").await.unwrap();
    assert_eq!(analytics.total_messages, 100);
    assert_eq!(analytics.unread_messages, 5);
    assert_eq!(analytics.messages_this_week, 15);
}

#[tokio::test]
async fn test_update_auto_responder() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("PATCH"))
        .and(path("/v1/agent-mailboxes/mbx_1/auto-responder"))
        .and(body_json(serde_json::json!({
            "enabled": true,
            "rules": [{ "match": "*", "action": { "reply_text": "Out of office" } }]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "auto_responder_enabled": true,
                "auto_responder_rules": [
                    { "match": "*", "action": { "reply_text": "Out of office" } }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    let config = client
        .update_auto_responder(
            "mbx_1",
            UpdateAutoResponderParams {
                enabled: Some(true),
                rules: Some(serde_json::json!([
                    { "match": "*", "action": { "reply_text": "Out of office" } }
                ])),
            },
        )
        .await
        .unwrap();

    assert!(config.auto_responder_enabled);
    assert_eq!(config.auto_responder_rules[0]["match"], "*");
}
