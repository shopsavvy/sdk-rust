//! Drives the scheduling methods through the real client against a local mock.
//! refinery's `schedule` / `unschedule` handlers read ONLY `req.query`, so the SDK
//! must send `PUT` / `DELETE /products/scheduled` with query params and no body.

use shopsavvy_sdk::{Client, Config, MonitoringFrequency};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const SCHEDULED_RESPONSE: &str = r#"{
  "success": true,
  "data": [
    {
      "title": "Keurig K-Mini Single Serve Coffee Maker, Black",
      "category": null,
      "brand": "Keurig",
      "color": "Black",
      "shopsavvy": "3ONn300xybP3y66ibqc1",
      "barcode": "611247373064",
      "amazon": "B07G14HTBZ",
      "model": "K-MINI",
      "mpn": null,
      "images": [],
      "schedule": "daily",
      "retailer": "amazon.com"
    },
    {
      "title": "Keurig K-Elite",
      "category": null,
      "brand": "Keurig",
      "color": null,
      "shopsavvy": "DrKWneG0MpFlZpwZXNYa",
      "barcode": "611247369449",
      "amazon": null,
      "model": "K-Elite",
      "mpn": null,
      "images": []
    }
  ],
  "meta": { "request_id": "req-1", "credits_used": 2, "credits_remaining": 998, "rate_limit_remaining": 999 }
}"#;

const UNSCHEDULE_RESPONSE: &str = r#"{
  "success": true,
  "message": "Products successfully removed from schedule",
  "meta": { "request_id": "req-2", "credits_used": 0, "credits_remaining": 0, "rate_limit_remaining": 0 }
}"#;

async fn setup(http_method: &str, body: &'static str) -> (MockServer, Client) {
    let server = MockServer::start().await;
    Mock::given(method(http_method))
        .and(path("/products/scheduled"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .expect(1)
        .mount(&server)
        .await;
    let client = Client::with_config(Config::new("ss_test_abc123").with_base_url(server.uri())).unwrap();
    (server, client)
}

/// The one request the mock received, with its decoded query pairs.
async fn only_request(server: &MockServer) -> (Request, Vec<(String, String)>) {
    let mut requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    let request = requests.remove(0);
    let pairs = request.url.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
    (request, pairs)
}

fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

#[tokio::test]
async fn schedule_single_sends_put_query_no_body() {
    let (server, client) = setup("PUT", SCHEDULED_RESPONSE).await;
    let response = client
        .schedule_product_monitoring("611247373064", MonitoringFrequency::Daily, None)
        .await
        .unwrap();

    let (request, query) = only_request(&server).await;
    assert_eq!(request.method.as_str(), "PUT");
    assert_eq!(request.url.path(), "/products/scheduled");
    assert_eq!(query, pairs(&[("ids", "611247373064"), ("schedule", "daily")]));
    assert!(request.body.is_empty(), "body must be empty, was {:?}", String::from_utf8_lossy(&request.body));

    assert!(response.success);
    assert_eq!(response.credits_used(), 2);
    assert_eq!(response.data.len(), 2);
    assert_eq!(response.data[0].product.title, "Keurig K-Mini Single Serve Coffee Maker, Black");
    assert_eq!(response.data[0].product.barcode.as_deref(), Some("611247373064"));
    assert_eq!(response.data[0].schedule.as_deref(), Some("daily"));
    assert_eq!(response.data[0].retailer.as_deref(), Some("amazon.com"));
    // Unlabelled interval: schedule and retailer both absent.
    assert_eq!(response.data[1].product.shopsavvy, "DrKWneG0MpFlZpwZXNYa");
    assert_eq!(response.data[1].schedule, None);
    assert_eq!(response.data[1].retailer, None);
}

#[tokio::test]
async fn schedule_single_with_retailer_sends_retailer_param() {
    let (server, client) = setup("PUT", SCHEDULED_RESPONSE).await;
    client
        .schedule_product_monitoring("611247373064", MonitoringFrequency::Hourly, Some("amazon.com"))
        .await
        .unwrap();

    let (request, query) = only_request(&server).await;
    assert_eq!(request.method.as_str(), "PUT");
    assert_eq!(request.url.path(), "/products/scheduled");
    assert_eq!(query, pairs(&[("ids", "611247373064"), ("schedule", "hourly"), ("retailer", "amazon.com")]));
    assert!(request.body.is_empty());
}

#[tokio::test]
async fn schedule_batch_comma_joins_ids() {
    let (server, client) = setup("PUT", SCHEDULED_RESPONSE).await;
    let response = client
        .schedule_product_monitoring_batch(&["611247373064", "611247369449"], MonitoringFrequency::Weekly, None)
        .await
        .unwrap();

    let (request, query) = only_request(&server).await;
    assert_eq!(request.method.as_str(), "PUT");
    assert_eq!(request.url.path(), "/products/scheduled");
    assert_eq!(query, pairs(&[("ids", "611247373064,611247369449"), ("schedule", "weekly")]));
    // Comma is percent-encoded on the wire.
    assert_eq!(request.url.query(), Some("ids=611247373064%2C611247369449&schedule=weekly"));
    assert!(request.body.is_empty());
    assert_eq!(response.data.len(), 2);
}

#[tokio::test]
async fn schedule_batch_with_retailer() {
    let (server, client) = setup("PUT", SCHEDULED_RESPONSE).await;
    client
        .schedule_product_monitoring_batch(&["611247373064", "B07G14HTBZ"], MonitoringFrequency::Daily, Some("bestbuy.com"))
        .await
        .unwrap();

    let (request, query) = only_request(&server).await;
    assert_eq!(request.method.as_str(), "PUT");
    assert_eq!(query, pairs(&[("ids", "611247373064,B07G14HTBZ"), ("schedule", "daily"), ("retailer", "bestbuy.com")]));
    assert!(request.body.is_empty());
}

#[tokio::test]
async fn unschedule_single_sends_delete_query_no_body() {
    let (server, client) = setup("DELETE", UNSCHEDULE_RESPONSE).await;
    let response = client.remove_product_from_schedule("611247373064").await.unwrap();

    let (request, query) = only_request(&server).await;
    assert_eq!(request.method.as_str(), "DELETE");
    assert_eq!(request.url.path(), "/products/scheduled");
    assert_eq!(query, pairs(&[("ids", "611247373064")]));
    assert!(request.body.is_empty());

    assert!(response.success);
    assert_eq!(response.message.as_deref(), Some("Products successfully removed from schedule"));
    assert_eq!(response.meta.unwrap().request_id.as_deref(), Some("req-2"));
}

#[tokio::test]
async fn unschedule_batch_comma_joins_ids() {
    let (server, client) = setup("DELETE", UNSCHEDULE_RESPONSE).await;
    let response = client
        .remove_products_from_schedule(&["611247373064", "611247369449"])
        .await
        .unwrap();

    let (request, query) = only_request(&server).await;
    assert_eq!(request.method.as_str(), "DELETE");
    assert_eq!(request.url.path(), "/products/scheduled");
    assert_eq!(query, pairs(&[("ids", "611247373064,611247369449")]));
    assert!(request.body.is_empty());
    assert!(response.success);
}

#[tokio::test]
async fn get_scheduled_products_parses_product_plus_schedule() {
    let (server, client) = setup("GET", SCHEDULED_RESPONSE).await;
    let response = client.get_scheduled_products().await.unwrap();

    let (request, query) = only_request(&server).await;
    assert_eq!(request.method.as_str(), "GET");
    assert!(query.is_empty());
    assert_eq!(response.data.len(), 2);
    assert_eq!(response.data[0].schedule.as_deref(), Some("daily"));
    assert_eq!(response.data[1].product.model.as_deref(), Some("K-Elite"));
}
