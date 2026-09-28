//! Drives `Client::get_price_history()` against a local mock of
//! `GET /products/offers/history`, serving a response of the exact shape the API
//! emits (refinery `offerHistory`: `data` is a list of products, each with
//! `offers`, each offer with a newest-first `history`).

use shopsavvy_sdk::{Client, Config, Error};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE: &str = include_str!("fixtures/price-history-response.json");

async fn client_for(server: &MockServer) -> Client {
    Client::with_config(Config::new("ss_test_abc123").with_base_url(server.uri())).unwrap()
}

#[tokio::test]
async fn get_price_history_sends_start_end_and_parses_products_offers_and_history() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/products/offers/history"))
        .and(query_param("ids", "611247373064,611247369449"))
        .and(query_param("start", "2022-11-20"))
        .and(query_param("end", "2022-11-27"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let response = client
        .get_price_history("611247373064,611247369449", "2022-11-20", "2022-11-27", None, None)
        .await
        .expect("a real-shaped price-history response must deserialize");

    // The old names must not be on the wire.
    let requests = server.received_requests().await.unwrap();
    let query = requests[0].url.query().unwrap_or_default().to_string();
    assert!(!query.contains("start_date"), "query was {query}");
    assert!(!query.contains("end_date"), "query was {query}");

    assert!(response.success);
    assert_eq!(response.credits_used(), 14);
    assert_eq!(response.credits_remaining(), 986);
    let meta = response.meta.as_ref().unwrap();
    assert_eq!(meta.request_id.as_deref(), Some("req-7f3c9a"));
    assert_eq!(meta.rate_limit_remaining, Some(999));

    assert_eq!(response.data.len(), 2);

    // Product 1: full product fields + two offers with history.
    let keurig = &response.data[0];
    assert_eq!(keurig.product.title, "Keurig K-Mini Single Serve Coffee Maker, Black");
    assert_eq!(keurig.product.shopsavvy, "3ONn300xybP3y66ibqc1");
    assert_eq!(keurig.product.barcode.as_deref(), Some("611247373064"));
    assert_eq!(keurig.product.amazon.as_deref(), Some("B07G14HTBZ"));
    assert_eq!(keurig.product.model.as_deref(), Some("K-MINI"));
    assert_eq!(keurig.product.title_short.as_deref(), Some("Keurig K-Mini"));
    assert_eq!(keurig.product.images.as_ref().unwrap().len(), 1);
    assert_eq!(keurig.product.rating.as_ref().unwrap()["count"], 51234);
    assert_eq!(keurig.offers.len(), 2);

    let amazon = &keurig.offers[0];
    assert_eq!(amazon.id, "0IUouCFtZEhxeOablTPl");
    assert_eq!(amazon.retailer.as_deref(), Some("Amazon"));
    assert_eq!(amazon.price, Some(74.96));
    assert_eq!(amazon.currency.as_deref(), Some("USD"));
    assert_eq!(amazon.condition.as_deref(), Some("new"));
    assert_eq!(amazon.seller.as_deref(), Some("ACME Deals"));
    assert_eq!(amazon.url.as_deref(), Some("https://www.amazon.com/dp/B07G14HTBZ?m=A1GKQADQC2VI6E"));
    assert_eq!(amazon.timestamp.as_deref(), Some("2022-11-27T22:36:33.236Z"));
    assert_eq!(amazon.history.len(), 3);
    assert_eq!(amazon.history[0].timestamp, "2022-11-27T22:36:33.236Z");
    assert_eq!(amazon.history[0].price, 74.96);
    assert_eq!(amazon.history[0].currency.as_deref(), Some("USD"));
    assert_eq!(amazon.history[0].availability.as_deref(), Some("in"));
    assert_eq!(amazon.history[1].price, 70.99);
    assert_eq!(amazon.history[1].availability.as_deref(), Some("out"));
    // Archived point with a null currency and no availability key at all.
    assert_eq!(amazon.history[2].price, 79.99);
    assert_eq!(amazon.history[2].timestamp, "2022-11-21T08:15:00.000Z");
    assert_eq!(amazon.history[2].currency, None);
    assert_eq!(amazon.history[2].availability, None);

    // Offer with availability omitted and seller: null.
    let best_buy = &keurig.offers[1];
    assert_eq!(best_buy.id, "Z9kQ2mBestBuyOffer01");
    assert_eq!(best_buy.retailer.as_deref(), Some("Best Buy"));
    assert_eq!(best_buy.availability, None);
    assert_eq!(best_buy.seller, None);
    assert_eq!(best_buy.price, Some(59.99));
    assert_eq!(best_buy.history.len(), 2);
    assert_eq!(best_buy.history[1].price, 64.99);

    // Product 2: nullable product fields, eBay offer with an empty history.
    let elite = &response.data[1];
    assert_eq!(elite.product.shopsavvy, "DrKWneG0MpFlZpwZXNYa");
    assert_eq!(elite.product.barcode.as_deref(), Some("611247369449"));
    assert_eq!(elite.product.category, None);
    assert_eq!(elite.product.color, None);
    assert_eq!(elite.product.amazon, None);
    assert_eq!(elite.product.mpn, None);
    assert_eq!(elite.product.images.as_deref(), Some(&[][..]));
    assert_eq!(elite.offers.len(), 1);
    assert_eq!(elite.offers[0].retailer.as_deref(), Some("eBay"));
    assert_eq!(elite.offers[0].seller.as_deref(), Some("coffee_reseller"));
    assert!(elite.offers[0].history.is_empty());
}

#[tokio::test]
async fn get_price_history_passes_retailer_filter() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/products/offers/history"))
        .and(query_param("ids", "611247373064"))
        .and(query_param("start", "2022-11-20"))
        .and(query_param("end", "2022-11-27"))
        .and(query_param("retailer", "amazon.com"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE, "application/json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let response = client
        .get_price_history("611247373064", "2022-11-20", "2022-11-27", Some("amazon.com"), None)
        .await
        .unwrap();
    assert_eq!(response.data[0].offers[0].history.len(), 3);
}

#[tokio::test]
async fn get_price_history_maps_api_errors_to_typed_errors() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/products/offers/history"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_raw(r#"{"success":false,"error":"A 'start' query parameter is required."}"#, "application/json"),
        )
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client
        .get_price_history("611247373064", "2022-11-20", "2022-11-27", None, None)
        .await
        .unwrap_err();
    match err {
        Error::Api { status_code, message } => {
            assert_eq!(status_code, 400);
            assert_eq!(message, "A 'start' query parameter is required.");
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}
