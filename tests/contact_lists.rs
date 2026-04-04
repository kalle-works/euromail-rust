use euromail::{AddContactParams, BulkAddContactsParams, CreateContactListParams, EuroMail};
use wiremock::matchers::{header, method, path};
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
