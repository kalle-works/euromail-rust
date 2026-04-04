use euromail::{AnalyticsQuery, DomainAnalyticsQuery, EuroMail, TimeseriesQuery};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_get_analytics_overview() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/analytics/overview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "sent": 1000,
                "delivered": 980,
                "bounced": 20,
                "opens": 500,
                "clicks": 100,
                "complaints": 2,
                "delivery_rate": 0.98,
                "open_rate": 0.51,
                "click_rate": 0.10
            },
            "period": {
                "from": "2026-02-07",
                "to": "2026-03-07",
                "period": "30d"
            }
        })))
        .mount(&mock_server)
        .await;

    let query = AnalyticsQuery {
        period: Some("30d".to_string()),
        from: None,
        to: None,
    };

    let overview = client.get_analytics_overview(Some(&query)).await.unwrap();
    assert_eq!(overview.period.from, "2026-02-07");
    assert_eq!(overview.data["sent"], 1000);
    assert_eq!(overview.data["delivery_rate"], 0.98);
}

#[tokio::test]
async fn test_get_analytics_timeseries() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/analytics/timeseries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "date": "2026-03-06",
                    "sent": 100,
                    "delivered": 98,
                    "bounced": 2,
                    "opens": 50,
                    "clicks": 10
                },
                {
                    "date": "2026-03-07",
                    "sent": 120,
                    "delivered": 118,
                    "bounced": 2,
                    "opens": 60,
                    "clicks": 15
                }
            ],
            "period": {
                "from": "2026-03-06",
                "to": "2026-03-07"
            }
        })))
        .mount(&mock_server)
        .await;

    let query = TimeseriesQuery {
        period: Some("7d".to_string()),
        from: None,
        to: None,
        metrics: None,
    };

    let result = client.get_analytics_timeseries(Some(&query)).await.unwrap();
    assert_eq!(result.data.len(), 2);
    assert_eq!(result.data[0].date, "2026-03-06");
    assert_eq!(result.data[0].sent, Some(100));
    assert_eq!(result.data[1].clicks, Some(15));
}

#[tokio::test]
async fn test_get_analytics_domains() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/v1/analytics/domains"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "domain": "example.com",
                    "sent": 500,
                    "delivered": 490,
                    "bounced": 10,
                    "open_rate": 0.45,
                    "click_rate": 0.12
                }
            ],
            "period": {
                "from": "2026-02-07",
                "to": "2026-03-07"
            }
        })))
        .mount(&mock_server)
        .await;

    let query = DomainAnalyticsQuery {
        period: None,
        from: None,
        to: None,
        limit: Some(10),
    };

    let result = client.get_analytics_domains(Some(&query)).await.unwrap();
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.data[0].domain, "example.com");
    assert_eq!(result.data[0].sent, 500);
    assert!((result.data[0].open_rate - 0.45).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_export_analytics_csv() {
    let mock_server = MockServer::start().await;
    let client = EuroMail::with_base_url("test-key", &mock_server.uri());

    let csv_content = "date,sent,delivered,bounced\n2026-03-06,100,98,2\n2026-03-07,120,118,2\n";

    Mock::given(method("GET"))
        .and(path("/v1/analytics/export"))
        .respond_with(ResponseTemplate::new(200).set_body_string(csv_content))
        .mount(&mock_server)
        .await;

    let result = client.export_analytics_csv(None).await.unwrap();
    assert!(result.contains("date,sent,delivered,bounced"));
    assert!(result.contains("2026-03-06,100,98,2"));
}
