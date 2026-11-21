use euromail::{EuroMail, UpdateDomainParams};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_domain_reads_detected_provider_beside_records() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/domains/dom-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "dom-1",
                "account_id": "acc-123",
                "domain": "example.com",
                "dkim_selector": "em1",
                "dkim_public_key": null,
                "spf_verified": false,
                "dkim_verified": false,
                "dmarc_verified": false,
                "return_path_verified": false,
                "mx_verified": false,
                "mx_verified_at": null,
                "inbound_enabled": false,
                "dns_records": {
                    "spf": {
                        "type": "TXT",
                        "host": "example.com",
                        "value": "v=spf1 include:euromail.dev ~all"
                    }
                },
                "detected_provider": "cloudflare",
                "verified_at": null,
                "created_at": "2026-10-28T00:00:00Z",
                "updated_at": "2026-10-28T00:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let domain = client.get_domain("dom-1").await.unwrap();
    assert_eq!(domain.detected_provider.as_deref(), Some("cloudflare"));
    assert_eq!(domain.dns_records["spf"].type_, "TXT");
}

fn domain_json(extra: serde_json::Value) -> serde_json::Value {
    let mut domain = serde_json::json!({
        "id": "dom-1",
        "account_id": "acc-123",
        "domain": "example.com",
        "dkim_selector": "em1",
        "dkim_public_key": null,
        "spf_verified": false,
        "dkim_verified": false,
        "dmarc_verified": false,
        "return_path_verified": false,
        "mx_verified": false,
        "mx_verified_at": null,
        "inbound_enabled": false,
        "dns_records": {},
        "verified_at": null,
        "created_at": "2026-11-21T00:00:00Z",
        "updated_at": "2026-11-21T00:00:00Z"
    });
    domain
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    serde_json::json!({ "data": domain })
}

#[tokio::test]
async fn test_update_domain_patches_only_require_tls_and_returns_it() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", mock_server.uri());

    Mock::given(method("PATCH"))
        .and(path("/v1/domains/dom-1"))
        .and(body_json(serde_json::json!({ "require_tls": true })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(domain_json(serde_json::json!({ "require_tls": true }))),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let domain = client
        .update_domain(
            "dom-1",
            &UpdateDomainParams {
                require_tls: Some(true),
            },
        )
        .await
        .unwrap();
    assert!(domain.require_tls);
}

#[tokio::test]
async fn test_domain_without_require_tls_parses_as_false() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/domains/dom-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(domain_json(serde_json::json!({}))))
        .mount(&mock_server)
        .await;

    let domain = client.get_domain("dom-1").await.unwrap();
    assert!(!domain.require_tls);
}
