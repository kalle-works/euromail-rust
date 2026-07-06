use euromail::{BroadcastParams, EuroMail};
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_send_broadcast_tracking_and_transactional() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/emails/broadcast"))
        .and(body_partial_json(serde_json::json!({
            "transactional": true,
            "tracking": false
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "data": {
                "operation_id": "op-1",
                "total_recipients": 3,
                "message": "queued"
            }
        })))
        .mount(&mock_server)
        .await;

    let params = BroadcastParams {
        contact_list_id: "cl_001".to_string(),
        from_address: "sender@example.com".to_string(),
        subject: Some("Migration notice".to_string()),
        text_body: Some("We moved!".to_string()),
        transactional: Some(true),
        tracking: Some(false),
        ..Default::default()
    };

    let response = client.send_broadcast(&params).await.unwrap();
    assert_eq!(response.total_recipients, 3);
}
