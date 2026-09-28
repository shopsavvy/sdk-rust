//! Feeds full fixtures of the exact JSON the Data API's `schedule` (PUT), `scheduled`
//! (GET) and `unschedule` (DELETE) handlers emit through the real client methods and
//! asserts every parsed value: `publicProductForProduct` fields + `schedule` / `retailer`,
//! and the `meta` block. Also covers `Offer::is_in_stock()` against the real availability
//! token (`"in"`).

use shopsavvy_sdk::{Client, Config, MonitoringFrequency};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PUT_FIXTURE: &str = include_str!("fixtures/schedule-put-fixture.json");
const LIST_FIXTURE: &str = include_str!("fixtures/schedule-list-fixture.json");
const DELETE_FIXTURE: &str = include_str!("fixtures/schedule-delete-fixture.json");

async fn serve(http_method: &str, endpoint: &str, body: &'static str) -> (MockServer, Client) {
    let server = MockServer::start().await;
    Mock::given(method(http_method))
        .and(path(endpoint))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .expect(1)
        .mount(&server)
        .await;
    let client = Client::with_config(Config::new("ss_test_abc123").with_base_url(server.uri())).unwrap();
    (server, client)
}

#[tokio::test]
async fn schedule_parses_full_product_fields_schedule_retailer_and_meta() {
    let (_server, client) = serve("PUT", "/products/scheduled", PUT_FIXTURE).await;
    let response = client
        .schedule_product_monitoring_batch(&["611247373064", "611247369449"], MonitoringFrequency::Daily, Some("amazon.com"))
        .await
        .unwrap();

    assert!(response.success);
    assert_eq!(response.data.len(), 2);

    let first = &response.data[0];
    let p = &first.product;
    assert_eq!(p.title, "Keurig K-Mini Single Serve Coffee Maker, Black");
    assert_eq!(p.shopsavvy, "3ONn300xybP3y66ibqc1");
    assert_eq!(p.category.as_deref(), Some("Kitchen & Dining"));
    assert_eq!(p.brand.as_deref(), Some("Keurig"));
    assert_eq!(p.color.as_deref(), Some("Black"));
    assert_eq!(p.barcode.as_deref(), Some("611247373064"));
    assert_eq!(p.amazon.as_deref(), Some("B07GV2S1GS"));
    assert_eq!(p.model.as_deref(), Some("K-Mini"));
    assert_eq!(p.mpn.as_deref(), Some("5000200237"));
    assert_eq!(p.images.as_ref().unwrap().len(), 2);
    assert_eq!(p.images.as_ref().unwrap()[1], "https://images.shopsavvy.com/3ONn300xybP3y66ibqc1/side.jpg");
    assert_eq!(p.title_short.as_deref(), Some("Keurig K-Mini"));
    assert_eq!(p.slug.as_deref(), Some("keurig-k-mini-single-serve-coffee-maker-black"));
    assert_eq!(p.description.as_deref(), Some("Brews any cup size between 6 and 12 oz."));
    assert_eq!(p.categories.as_ref().unwrap(), &vec!["Home & Kitchen".to_string(), "Coffee Makers".to_string()]);
    assert_eq!(p.attributes.as_ref().unwrap().get("Wattage").map(String::as_str), Some("600 W"));
    let rating = p.rating.as_ref().unwrap();
    assert_eq!(rating["value"].as_f64(), Some(4.6));
    assert_eq!(rating["count"].as_i64(), Some(18234));
    let score = p.score.as_ref().unwrap();
    assert_eq!(score["overall"].as_f64(), Some(0.82));
    assert_eq!(score["aspects"]["reliability"].as_f64(), Some(0.7));
    assert_eq!(p.keywords.as_ref().unwrap().len(), 2);
    assert_eq!(p.identifiers.as_ref().unwrap()["walmart"].as_str(), Some("592453934"));
    assert_eq!(first.schedule.as_deref(), Some("daily"));
    assert_eq!(first.retailer.as_deref(), Some("amazon.com"));

    // Second product: JSON-null and absent optional fields decode to None.
    let second = &response.data[1];
    let p2 = &second.product;
    assert_eq!(p2.shopsavvy, "DrKWneG0MpFlZpwZXNYa");
    assert_eq!(p2.category, None);
    assert_eq!(p2.color, None);
    assert_eq!(p2.mpn, None);
    assert_eq!(p2.amazon, None);
    assert!(p2.rating.is_none() && p2.score.is_none() && p2.identifiers.is_none());
    assert_eq!(p2.images.as_ref().unwrap().len(), 0);
    assert_eq!(second.schedule.as_deref(), Some("daily"));
    assert_eq!(second.retailer.as_deref(), Some("amazon.com"));

    let meta = response.meta.as_ref().unwrap();
    assert_eq!(meta.request_id.as_deref(), Some("req_put_7f3a"));
    assert_eq!(response.credits_used(), 2);
    assert_eq!(response.credits_remaining(), 998);
    assert_eq!(meta.rate_limit_remaining, Some(59));
    assert_eq!(response.message, None);
}

#[tokio::test]
async fn get_scheduled_products_parses_schedule_and_retailer_including_omitted_keys() {
    let (_server, client) = serve("GET", "/products/scheduled", LIST_FIXTURE).await;
    let response = client.get_scheduled_products().await.unwrap();

    assert!(response.success);
    assert_eq!(response.data.len(), 3);

    let first = &response.data[0];
    assert_eq!(first.product.shopsavvy, "3ONn300xybP3y66ibqc1");
    assert_eq!(first.product.barcode.as_deref(), Some("611247373064"));
    assert_eq!(first.product.rating.as_ref().unwrap()["count"].as_i64(), Some(18234));
    assert_eq!(first.schedule.as_deref(), Some("hourly"));
    assert_eq!(first.retailer.as_deref(), Some("bestbuy.com"));

    // Scheduled across every retailer: no `retailer` key.
    let second = &response.data[1];
    assert_eq!(second.product.model.as_deref(), Some("K-Elite"));
    assert_eq!(second.product.category, None);
    assert_eq!(second.schedule.as_deref(), Some("weekly"));
    assert_eq!(second.retailer, None);

    // Saved at an interval with no Data API label (e.g. 4h via ShopSavvy Business):
    // the server omits `schedule` entirely.
    let third = &response.data[2];
    assert_eq!(third.product.shopsavvy, "Qm1bXz0AirPodsPro02");
    assert_eq!(third.product.title, "Apple AirPods Pro (2nd generation)");
    assert_eq!(third.schedule, None);
    assert_eq!(third.retailer, None);

    let meta = response.meta.as_ref().unwrap();
    assert_eq!(meta.request_id.as_deref(), Some("req_list_91bc"));
    assert_eq!(meta.credits_used, 0);
    assert_eq!(meta.rate_limit_remaining, Some(0));
}

#[tokio::test]
async fn unschedule_parses_success_message_and_meta_without_data() {
    let (_server, client) = serve("DELETE", "/products/scheduled", DELETE_FIXTURE).await;
    let response = client
        .remove_products_from_schedule(&["611247373064", "611247369449"])
        .await
        .unwrap();

    assert!(response.success);
    assert_eq!(response.message.as_deref(), Some("Products successfully removed from schedule"));
    let meta = response.meta.unwrap();
    assert_eq!(meta.request_id.as_deref(), Some("req_del_22d0"));
    assert_eq!(meta.credits_used, 0);
    assert_eq!(meta.credits_remaining, 0);
    assert_eq!(meta.rate_limit_remaining, Some(0));
}

#[tokio::test]
async fn offer_is_in_stock_matches_the_api_availability_token() {
    // Offers exactly as publicOfferForOfferDoc emits them: availability "in" / "out",
    // and the key omitted when the stored value is "unknown".
    const OFFERS: &str = r#"{
      "success": true,
      "data": [{
        "title": "Keurig K-Mini", "shopsavvy": "3ONn300xybP3y66ibqc1", "barcode": "611247373064", "images": [],
        "offers": [
          {"id": "o1", "availability": "in", "condition": "new", "retailer": "Amazon", "currency": "USD", "price": 79.99, "seller": null, "URL": "https://www.amazon.com/dp/B07GV2S1GS", "timestamp": "2026-09-20T10:00:00.000Z", "history": []},
          {"id": "o2", "availability": "out", "condition": "new", "retailer": "Target", "currency": "USD", "price": 89.99, "URL": "https://www.target.com/p/-/A-53766836", "timestamp": "2026-09-20T10:00:00.000Z", "history": []},
          {"id": "o3", "condition": "used", "retailer": "eBay", "currency": "USD", "price": 45.0, "seller": "coffee_resale", "URL": "https://www.ebay.com/itm/1", "timestamp": "2026-09-20T10:00:00.000Z", "history": []}
        ]
      }],
      "meta": {"request_id": "req_off_1", "credits_used": 3, "credits_remaining": 995, "rate_limit_remaining": 58}
    }"#;
    let (_server, client) = serve("GET", "/products/offers", OFFERS).await;
    let response = client.get_current_offers("611247373064", None, None).await.unwrap();

    let offers = &response.data[0].offers;
    assert_eq!(offers.len(), 3);
    assert!(offers[0].is_in_stock());
    assert!(!offers[1].is_in_stock());
    assert_eq!(offers[2].availability, None);
    assert!(!offers[2].is_in_stock());
    assert_eq!(offers.iter().filter(|o| o.is_in_stock()).count(), 1);
}
