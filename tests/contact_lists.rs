use euromail::{
    AddContactParams, BulkAddContactsParams, ConfigureWelcomeEmailParams, CreateContactListParams,
    EuroMail, EuroMailError,
};
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_create_contact_list() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/contact-lists"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "cl-100",
                "account_id": "acc-123",
                "name": "Newsletter Subscribers",
                "description": "Main list",
                "double_opt_in": true,
                "contact_count": 0,
                "created_at": "2026-03-07T12:00:00Z",
                "updated_at": "2026-03-07T12:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let params = CreateContactListParams {
        name: "Newsletter Subscribers".to_string(),
        description: Some("Main list".to_string()),
        double_opt_in: Some(true),
    };

    let list = client.create_contact_list(&params).await.unwrap();
    assert_eq!(list.id, "cl-100");
    assert_eq!(list.name, "Newsletter Subscribers");
    assert!(list.double_opt_in);
    assert_eq!(list.contact_count, 0);
}

#[tokio::test]
async fn test_list_contact_lists() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/contact-lists"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "cl-100",
                    "account_id": "acc-123",
                    "name": "Newsletter",
                    "description": null,
                    "double_opt_in": false,
                    "contact_count": 150,
                    "created_at": "2026-03-07T12:00:00Z",
                    "updated_at": "2026-03-07T12:00:00Z"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let lists = client.list_contact_lists().await.unwrap();
    assert_eq!(lists.len(), 1);
    assert_eq!(lists[0].name, "Newsletter");
    assert_eq!(lists[0].contact_count, 150);
}

#[tokio::test]
async fn test_add_contact() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/contact-lists/cl-100/contacts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "ct-500",
                "list_id": "cl-100",
                "email": "user@example.com",
                "metadata": null,
                "status": "active",
                "created_at": "2026-03-07T12:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let params = AddContactParams {
        email: "user@example.com".to_string(),
        metadata: None,
    };

    let contact = client.add_contact("cl-100", &params).await.unwrap();
    assert_eq!(contact.id, "ct-500");
    assert_eq!(contact.email, "user@example.com");
    assert_eq!(contact.status, "active");
}

#[tokio::test]
async fn test_bulk_add_contacts() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/v1/contact-lists/cl-100/contacts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "inserted": 3,
                "total_requested": 5
            }
        })))
        .mount(&mock_server)
        .await;

    let params = BulkAddContactsParams {
        contacts: vec![
            AddContactParams {
                email: "a@example.com".to_string(),
                metadata: None,
            },
            AddContactParams {
                email: "b@example.com".to_string(),
                metadata: None,
            },
        ],
    };

    let response = client.bulk_add_contacts("cl-100", &params).await.unwrap();
    assert_eq!(response.inserted, 3);
    assert_eq!(response.total_requested, 5);
}

#[tokio::test]
async fn test_delete_contact_list() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("DELETE"))
        .and(path("/v1/contact-lists/cl-100"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    client.delete_contact_list("cl-100").await.unwrap();
}

#[tokio::test]
async fn test_get_welcome_email_unconfigured() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/contact-lists/cl-100/welcome-email"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "enabled": false,
                "subject": null,
                "html_body": null,
                "text_body": null,
                "template_id": null,
                "from_address": null,
                "delay_seconds": 0
            }
        })))
        .mount(&mock_server)
        .await;

    let config = client.get_welcome_email("cl-100").await.unwrap();
    assert!(!config.enabled);
    assert_eq!(config.subject, None);
    assert_eq!(config.delay_seconds, 0);
}

#[tokio::test]
async fn test_configure_welcome_email_with_inline_body() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("PUT"))
        .and(path("/v1/contact-lists/cl-100/welcome-email"))
        .and(body_partial_json(serde_json::json!({
            "enabled": true,
            "subject": "Welcome!",
            "html_body": "<h1>Hi</h1>",
            "delay_seconds": 120
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "cl-100",
                "account_id": "acc-123",
                "name": "Newsletter",
                "description": null,
                "double_opt_in": false,
                "contact_count": 0,
                "created_at": "2026-03-07T12:00:00Z",
                "updated_at": "2026-04-23T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let list = client
        .configure_welcome_email(
            "cl-100",
            &ConfigureWelcomeEmailParams::new()
                .enable()
                .subject("Welcome!")
                .html_body("<h1>Hi</h1>")
                .delay_seconds(120),
        )
        .await
        .unwrap();

    assert_eq!(list.id, "cl-100");
    assert_eq!(list.updated_at, "2026-04-23T10:00:00Z");
}

#[tokio::test]
async fn test_configure_welcome_email_with_template() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("PUT"))
        .and(path("/v1/contact-lists/cl-100/welcome-email"))
        .and(body_partial_json(serde_json::json!({
            "enabled": true,
            "template_id": "tpl-welcome"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "cl-100",
                "account_id": "acc-123",
                "name": "Newsletter",
                "description": null,
                "double_opt_in": false,
                "contact_count": 0,
                "created_at": "2026-03-07T12:00:00Z",
                "updated_at": "2026-04-23T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    client
        .configure_welcome_email(
            "cl-100",
            &ConfigureWelcomeEmailParams::new()
                .enable()
                .template_id("tpl-welcome"),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_configure_welcome_email_disable() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("PUT"))
        .and(path("/v1/contact-lists/cl-100/welcome-email"))
        .and(body_partial_json(serde_json::json!({ "enabled": false })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "cl-100",
                "account_id": "acc-123",
                "name": "Newsletter",
                "description": null,
                "double_opt_in": false,
                "contact_count": 0,
                "welcome_email_enabled": false,
                "welcome_email_subject": null,
                "welcome_email_html_body": null,
                "welcome_email_text_body": null,
                "welcome_email_template_id": null,
                "welcome_email_from_address": null,
                "welcome_email_delay_seconds": 0,
                "created_at": "2026-03-07T12:00:00Z",
                "updated_at": "2026-04-23T10:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let list = client
        .configure_welcome_email("cl-100", &ConfigureWelcomeEmailParams::new().disable())
        .await
        .unwrap();

    assert!(!list.welcome_email_enabled);
    assert_eq!(list.welcome_email_subject, None);
}

#[tokio::test]
async fn test_configure_welcome_email_validation_error() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("PUT"))
        .and(path("/v1/contact-lists/cl-100/welcome-email"))
        .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
            "code": "invalid_params",
            "message": "welcome email requires either template_id or html_body/text_body when enabled"
        })))
        .mount(&mock_server)
        .await;

    let result = client
        .configure_welcome_email("cl-100", &ConfigureWelcomeEmailParams::new().enable())
        .await;

    match result {
        Err(EuroMailError::Validation { code, message }) => {
            assert_eq!(code, "invalid_params");
            assert!(message.contains("template_id"));
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}
