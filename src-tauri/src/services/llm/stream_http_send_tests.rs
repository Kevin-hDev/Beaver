use std::time::Duration;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::request_purpose::RequestPurpose;
use super::{route, stream_http_send};

#[tokio::test]
async fn emitted_xai_oauth_request_carries_the_model_route_header() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .and(header("x-grok-model-override", "grok-4.6"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let client =
        crate::services::secure_http::AuthenticatedClient::new_loopback(Duration::from_secs(2))
            .expect("client");
    let route = route::resolve("xai-oauth").unwrap();
    let headers =
        stream_http_send::outbound_headers(&route, "grok-4.6", None, RequestPurpose::ManualChat)
            .unwrap();

    let response = client
        .send(
            client
                .post(format!("{}/chat", server.uri()))
                .headers(headers)
                .bearer_auth("fixture-secret")
                .json(&serde_json::json!({"model": "grok-4.6"})),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[test]
fn xai_api_requests_do_not_receive_subscription_routing_headers() {
    let route = route::resolve("xai").unwrap();

    let headers =
        stream_http_send::outbound_headers(&route, "grok-4.6", None, RequestPurpose::ManualChat)
            .unwrap();

    assert!(!headers.contains_key("x-grok-model-override"));
}
