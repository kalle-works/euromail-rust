use euromail::EuroMail;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_account() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/account"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "acc-123",
                "name": "Test Account",
                "email": "test@example.com",
                "plan": "pro",
                "monthly_quota": 100000,
                "emails_sent_this_month": 4500,
                "quota_reset_at": "2026-04-01T00:00:00Z",
                "created_at": "2025-01-15T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let account = client.get_account().await.unwrap();
    assert_eq!(account.id, "acc-123");
    assert_eq!(account.name, "Test Account");
    assert_eq!(account.email, "test@example.com");
    assert_eq!(account.plan, "pro");
    assert_eq!(account.monthly_quota, 100000);
    assert_eq!(account.emails_sent_this_month, 4500);
}
