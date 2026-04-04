use euromail::{CreateApiKeyParams, EuroMail};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_create_api_key() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/api-keys"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "key-123",
                "name": "My Key",
                "key": "em_live_abc123def456",
                "key_prefix": "em_live_abc1",
                "scopes": ["emails:send", "emails:read"],
                "created_at": "2026-03-16T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let params = CreateApiKeyParams {
        name: "My Key".to_string(),
        scopes: Some(vec!["emails:send".to_string(), "emails:read".to_string()]),
    };

    let response = client.create_api_key(&params).await.unwrap();
    assert_eq!(response.id, "key-123");
    assert_eq!(response.name, "My Key");
    assert!(response.key.starts_with("em_live_"));
    assert_eq!(response.scopes.len(), 2);
}

#[tokio::test]
async fn test_list_api_keys() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/api-keys"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "key-1",
                    "name": "Production",
                    "key_prefix": "em_live_abc1",
                    "scopes": ["emails:send"],
                    "last_used_at": "2026-03-15T14:30:00Z",
                    "is_active": true,
                    "created_at": "2026-03-01T10:00:00Z"
                },
                {
                    "id": "key-2",
                    "name": "Read Only",
                    "key_prefix": "em_live_def2",
                    "scopes": ["emails:read"],
                    "last_used_at": null,
                    "is_active": true,
                    "created_at": "2026-03-10T08:00:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let keys = client.list_api_keys().await.unwrap();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0].name, "Production");
    assert_eq!(keys[1].name, "Read Only");
    assert!(keys[0].last_used_at.is_some());
    assert!(keys[1].last_used_at.is_none());
}

#[tokio::test]
async fn test_delete_api_key() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("DELETE"))
        .and(path("/v1/api-keys/key-123"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    client.delete_api_key("key-123").await.unwrap();
}
