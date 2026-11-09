//! `GET /products/offers` carries, per product, an `in_store_prices` array: shelf prices read
//! from shoppers' scanned receipts, labeled `source: "user-collected"`.

use shopsavvy_sdk::{Client, Config};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE: &str = include_str!("fixtures/offers-in-store-response.json");

async fn client_for(server: &MockServer) -> Client {
    Client::with_config(Config::new("ss_test_abc123").with_base_url(server.uri())).unwrap()
}

#[tokio::test]
async fn get_current_offers_parses_user_collected_in_store_prices() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/products/offers"))
        .and(query_param("ids", "011110838360"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let response = client_for(&server).await.get_current_offers("011110838360", None, None).await.unwrap();
    let product = &response.data[0];
    assert_eq!(product.in_store_prices.len(), 1);
    let in_store = &product.in_store_prices[0];
    assert_eq!(in_store.source, "user-collected");
    assert_eq!(in_store.retailer.as_deref(), Some("Kroger"));
    assert_eq!(in_store.store.address.as_deref(), Some("1014 Vine St, Cincinnati, OH 45202, USA"));
    assert_eq!(in_store.store.latitude, Some(39.1086));
    assert_eq!(in_store.price, 3.49);
    assert_eq!(in_store.observed_on.as_deref(), Some("2026-11-07"));
    assert_eq!(product.offers[0].retailer.as_deref(), Some("Walmart"));
}

#[tokio::test]
async fn in_store_prices_default_to_empty_when_omitted() {
    let mut body: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    body["data"][0].as_object_mut().unwrap().remove("in_store_prices");
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/products/offers"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body.to_string(), "application/json"))
        .mount(&server)
        .await;

    let response = client_for(&server).await.get_current_offers("011110838360", None, None).await.unwrap();
    assert!(response.data[0].in_store_prices.is_empty());
}
