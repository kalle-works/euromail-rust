use euromail::EuroMail;
use wiremock::matchers::{method, path};
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
