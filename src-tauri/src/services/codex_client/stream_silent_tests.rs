use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use super::*;
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::provider_usage::{
    RequestMeasurement, RequestMeasurementContext, UsageApiFormat, UsageWorkload,
};
use crate::services::secure_http::AuthenticatedClient;

async fn oversized_incomplete_sse_response() -> (reqwest::Response, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0_u8; 1024];
        let _ = socket.read(&mut request).await;
        let _ = socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\n")
            .await;
        let _ = socket.write_all(b"data: ").await;
        let body = vec![b'x'; crate::services::secure_http::LLM_BODY_LIMIT + 1];
        let _ = socket.write_all(&body).await;
        tokio::time::sleep(Duration::from_secs(10)).await;
    });
    let client = AuthenticatedClient::new_loopback(Duration::from_secs(2)).unwrap();
    let response = client
        .send(client.get(format!("http://{address}/stream")))
        .await
        .unwrap();
    (response, server)
}

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
        "openai",
        "gpt-5.6-sol",
        &mut measurement,
    )
    .await
    .unwrap_err();

    assert_eq!(error, "provider_request_rejected");
    server.await.unwrap();
}

#[tokio::test]
async fn silent_oversized_incomplete_sse_is_rejected_before_the_idle_deadline() {
    let (response, server) = oversized_incomplete_sse_response().await;
    let mut measurement =
        crate::services::codex_client::stream_measurement::StreamMeasurement::new(None);
    let error = consume_sse_silent(
        response,
        CancellationToken::new(),
        Duration::from_secs(5),
        None,
        "openai",
        "gpt-5.6-sol",
        &mut measurement,
    )
    .await
    .unwrap_err();

    assert_eq!(error, "provider_connection_failed");
    server.abort();
}

#[tokio::test]
async fn silent_responses_consumer_observes_the_final_served_tier() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let body =
        "data: {\"type\":\"response.completed\",\"response\":{\"service_tier\":\"default\"}}\n\n";
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
    let mut request_measurement = RequestMeasurement::start(RequestMeasurementContext {
        connection_id: "codex-oauth",
        canonical_provider_id: "openai",
        api_format: UsageApiFormat::Responses,
        model: "gpt-5.6-sol",
        session_id: Some("session-1"),
        request_id: "request-1",
        turn: Some(1),
        attempt: 1,
        workload: UsageWorkload::Compression,
        fast_mode: FastModeRequest::Fast,
    })
    .unwrap();
    let mut measurement = crate::services::codex_client::stream_measurement::StreamMeasurement::new(
        Some(&mut request_measurement),
    );

    consume_sse_silent(
        response,
        CancellationToken::new(),
        Duration::from_secs(1),
        None,
        "openai",
        "gpt-5.6-sol",
        &mut measurement,
    )
    .await
    .unwrap();

    assert_eq!(
        request_measurement.fast_observation().1,
        crate::services::provider_usage::ServiceTierServed::Default
    );
    server.await.unwrap();
}
