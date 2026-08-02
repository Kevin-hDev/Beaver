use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use super::*;
use crate::services::secure_http::AuthenticatedClient;

#[test]
fn local_output_limit_is_optional() {
    let result = StreamResult {
        content: "x".repeat(100),
        ..Default::default()
    };
    assert!(!output_is_over_local_limit(&result, None));
}

#[test]
fn local_output_limit_uses_safe_char_estimate() {
    let result = StreamResult {
        content: "x".repeat(60),
        ..Default::default()
    };
    assert!(output_is_over_local_limit(&result, Some(10)));
}

#[tokio::test]
async fn silent_stream_rejects_the_generic_error_event_immediately() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let body = "data: {\"type\":\"error\",\"error\":{\"code\":\"invalid_request\"}}\n\n";
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0_u8; 1024];
        let _ = socket.read(&mut request).await;
        let headers = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        socket.write_all(headers.as_bytes()).await.unwrap();
        socket.write_all(body.as_bytes()).await.unwrap();
    });
    let client = AuthenticatedClient::new_loopback(Duration::from_secs(2)).unwrap();
    let response = client
        .send(client.get(format!("http://{address}/stream")))
        .await
        .unwrap();
    let mut measurement =
        crate::services::codex_client::stream_measurement::StreamMeasurement::new(None);

    let error = consume_sse_silent(
        response,
        CancellationToken::new(),
        Duration::from_secs(1),
        None,
        "gpt-5.6-sol",
        &mut measurement,
    )
    .await
    .unwrap_err();

    assert_eq!(error, "provider_request_rejected");
    server.await.unwrap();
}
