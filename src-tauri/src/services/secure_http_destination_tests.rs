use std::net::SocketAddr;
use std::time::Duration;

use super::FixedDestination;
use crate::services::secure_http::SecureHttpError;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn pinned_destination_refuses_a_different_url_before_send() {
    let fixed = FixedDestination::new(
        "https://example.com/a".parse().unwrap(),
        SocketAddr::from(([1, 1, 1, 1], 443)),
        Duration::from_millis(200),
    )
    .unwrap();
    let request = fixed.client.get("https://example.com/b");
    assert_eq!(
        fixed.send(request).await.unwrap_err(),
        SecureHttpError::InsecureUrl
    );
}

#[tokio::test]
async fn pinned_destination_never_follows_a_redirect() {
    let target = MockServer::start().await;
    let origin = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(302).insert_header("Location", format!("{}/sink", target.uri())),
        )
        .mount(&origin)
        .await;
    let fixed = FixedDestination::new_loopback(
        format!("{}/start", origin.uri()).parse().unwrap(),
        Duration::from_secs(2),
    )
    .unwrap();
    assert_eq!(
        fixed.send(fixed.get()).await.unwrap_err(),
        SecureHttpError::Redirect
    );
    assert!(target.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn pinned_destination_uses_checked_address_not_a_second_dns_lookup() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    let address = server.address();
    let fixed = FixedDestination::new_test_pinned(
        format!("http://oauth.fixture.invalid:{}/token", address.port())
            .parse()
            .unwrap(),
        *address,
        Duration::from_secs(2),
    )
    .unwrap();
    let response = fixed.send(fixed.get()).await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}
