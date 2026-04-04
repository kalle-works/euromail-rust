use euromail::EuroMail;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_gdpr_export_email() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/gdpr/export"))
        .and(query_param("email", "user@example.com"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "email_address": "user@example.com",
                "emails": [
                    {"id": "email-1", "subject": "Welcome", "status": "delivered"}
                ],
                "events": [
                    {"id": "evt-1", "event_type": "delivered"}
                ],
                "suppressions": [],
                "unsubscribe_events": [],
                "inbound_emails": []
            },
            "exported_at": "2026-03-16T10:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let response = client.gdpr_export_email("user@example.com").await.unwrap();
    assert_eq!(response.data.email_address, "user@example.com");
    assert_eq!(response.data.emails.len(), 1);
    assert_eq!(response.data.events.len(), 1);
    assert!(response.data.suppressions.is_empty());
}

#[tokio::test]
async fn test_gdpr_erase_email() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("DELETE"))
        .and(path("/v1/gdpr/erase"))
        .and(query_param("email", "user@example.com"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "email_address": "user@example.com",
                "rows_deleted": 42,
                "message": "All data for this email address has been permanently deleted."
            }
        })))
        .mount(&mock_server)
        .await;

    let response = client.gdpr_erase_email("user@example.com").await.unwrap();
    assert_eq!(response.data.email_address, "user@example.com");
    assert_eq!(response.data.rows_deleted, 42);
}
