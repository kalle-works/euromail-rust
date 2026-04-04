use euromail::{EuroMail, EuroMailError};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_authentication_error() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("bad-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/account"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "code": "unauthorized",
            "message": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let result = client.get_account().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EuroMailError::Authentication(msg) => {
            assert_eq!(msg, "Invalid API key");
        }
        other => panic!("Expected Authentication error, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_not_found_error() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/emails/nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "code": "not_found",
            "message": "Email not found"
        })))
        .mount(&mock_server)
        .await;

    let result = client.get_email("nonexistent").await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EuroMailError::NotFound(msg) => {
            assert_eq!(msg, "Email not found");
        }
        other => panic!("Expected NotFound error, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_validation_error() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/emails"))
        .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
            "code": "invalid_params",
            "message": "Missing required field: to"
        })))
        .mount(&mock_server)
        .await;

    let params = euromail::SendEmailParams {
        from: "sender@example.com".to_string(),
        to: euromail::Recipient::One(String::new()),
        subject: Some("Test".to_string()),
        html_body: None,
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

    let result = client.send_email(&params).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EuroMailError::Validation { code, message } => {
            assert_eq!(code, "invalid_params");
            assert_eq!(message, "Missing required field: to");
        }
        other => panic!("Expected Validation error, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_rate_limit_error() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/account"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "30")
                .set_body_json(serde_json::json!({
                    "code": "rate_limited",
                    "message": "Too many requests"
                })),
        )
        .mount(&mock_server)
        .await;

    let result = client.get_account().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EuroMailError::RateLimit {
            retry_after,
            message,
        } => {
            assert_eq!(retry_after, Some(30));
            assert_eq!(message, "Too many requests");
        }
        other => panic!("Expected RateLimit error, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_server_error() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/account"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "code": "internal_error",
            "message": "Internal server error"
        })))
        .mount(&mock_server)
        .await;

    let result = client.get_account().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        EuroMailError::Api {
            status,
            code,
            message,
        } => {
            assert_eq!(status, 500);
            assert_eq!(code, "internal_error");
            assert_eq!(message, "Internal server error");
        }
        other => panic!("Expected Api error, got: {other:?}"),
    }
}
