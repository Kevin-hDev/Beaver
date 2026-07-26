use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;

#[tokio::test]
async fn default_client_rejects_plain_http_before_sending() {
    let server = MockServer::start().await;
    let client = AuthenticatedClient::new(Duration::from_secs(2)).unwrap();

    let result = client.send(client.get(server.uri())).await;

    assert!(result.is_err());
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn loopback_client_rejects_non_loopback_plain_http() {
    let client = AuthenticatedClient::new_loopback(Duration::from_secs(2)).unwrap();

    let result = client.send(client.get("http://example.com/private")).await;

    assert_eq!(result.unwrap_err(), SecureHttpError::InsecureUrl);
}

#[tokio::test]
async fn redirects_never_forward_credentials_or_bodies() {
    for status in [302, 307, 308] {
        let destination = MockServer::start().await;
        let origin = MockServer::start().await;
        Mock::given(any())
            .respond_with(
                ResponseTemplate::new(status)
                    .insert_header("Location", format!("{}/sink", destination.uri())),
            )
            .mount(&origin)
            .await;

        let client = AuthenticatedClient::new_loopback(Duration::from_secs(2)).unwrap();
        let request = client
            .post(format!("{}/start", origin.uri()))
            .bearer_auth("fixture-credential")
            .body("fixture-body");
        assert!(client.send(request).await.is_err());
        assert!(destination.received_requests().await.unwrap().is_empty());
    }
}

#[tokio::test]
async fn chunked_response_without_length_is_stopped_at_the_limit() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0u8; 1024];
        let _ = socket.read(&mut request).await;
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n8\r\n12345678\r\n8\r\nabcdefgh\r\n0\r\n\r\n")
            .await
            .unwrap();
    });

    let client = AuthenticatedClient::new_loopback(Duration::from_secs(2)).unwrap();
    let response = client
        .send(client.get(format!("http://{address}/body")))
        .await
        .unwrap();
    assert!(read_bounded(response, 10).await.is_err());
    server.await.unwrap();
}

#[tokio::test]
async fn errors_never_echo_request_details() {
    let client = AuthenticatedClient::new(Duration::from_millis(100)).unwrap();
    let fixture = "credential-fixture";
    let error = client
        .send(
            client
                .get("http://127.0.0.1:1/private-path")
                .bearer_auth(fixture),
        )
        .await
        .unwrap_err()
        .to_string();
    assert!(!error.contains(fixture));
    assert!(!error.contains("127.0.0.1"));
    assert!(!error.contains("private-path"));
}

#[tokio::test]
async fn forecast_limit_accepts_a_small_loopback_response() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
        .mount(&server)
        .await;
    let client = AuthenticatedClient::new_loopback(Duration::from_secs(2)).unwrap();
    let response = client.send(client.get(server.uri())).await.unwrap();

    let body: serde_json::Value = read_json_bounded(
        response,
        crate::services::forecast::limits::MAX_RESPONSE_BYTES,
    )
    .await
    .unwrap();

    assert_eq!(body["ok"], true);
}
