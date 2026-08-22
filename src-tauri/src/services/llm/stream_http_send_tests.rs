use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue};
use wiremock::matchers::{header, header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::request_purpose::RequestPurpose;
use super::{route, stream_http_send};

#[tokio::test]
async fn emitted_request_merges_auth_and_outbound_headers() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .and(header("x-authenticateresponse", "authenticate-response"))
        .and(header("x-grok-model-override", "grok-4.6"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let client =
        crate::services::secure_http::AuthenticatedClient::new_loopback(Duration::from_secs(2))
            .expect("client");
    let mut auth_headers = HeaderMap::new();
    auth_headers.insert(
        "x-authenticateresponse",
        HeaderValue::from_static("authenticate-response"),
    );
    let mut outbound_headers = HeaderMap::new();
    outbound_headers.insert(
        "x-grok-model-override",
        HeaderValue::from_static("grok-4.6"),
    );

    let request = stream_http_send::json_request_builder(
        &client,
        &format!("{}/chat", server.uri()),
        &serde_json::json!({"model": "grok-4.6"}),
        "fixture-secret",
        auth_headers,
        &outbound_headers,
    );
    let response = client.send(request).await.unwrap();

    assert_eq!(response.status(), 200);
}

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
    let route = route::test_route("xai-oauth");
    let response = stream_http_send::send_json_request(
        &client,
        &route,
        &format!("{}/chat", server.uri()),
        &serde_json::json!({"model": "grok-4.6"}),
        RequestPurpose::ManualChat,
        "grok-4.6",
        None,
    )
    .await
    .unwrap();

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn emitted_xai_api_request_carries_the_conversation_cache_header() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat"))
        .and(header_exists("x-grok-conv-id"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let client =
        crate::services::secure_http::AuthenticatedClient::new_loopback(Duration::from_secs(2))
            .expect("client");
    let route = route::test_route("xai");

    let response = stream_http_send::send_json_request(
        &client,
        &route,
        &format!("{}/chat", server.uri()),
        &serde_json::json!({"model": "grok-4.6"}),
        RequestPurpose::ManualChat,
        "grok-4.6",
        Some("session-fixture"),
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
