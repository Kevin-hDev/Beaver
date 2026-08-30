use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use super::*;
use crate::services::agent_local::stream_buffer::StreamEventSink;
use crate::services::agent_local::types_ollama::{StreamEvent, StreamOutcome};
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::provider_usage::{
    RequestMeasurement, RequestMeasurementContext, UsageApiFormat, UsageWorkload,
};
use crate::services::secure_http::AuthenticatedClient;

struct NoopSink;

impl StreamEventSink for NoopSink {
    fn send_event(&self, _event: StreamEvent) -> Result<(), String> {
        Ok(())
    }
}

fn request_measurement() -> RequestMeasurement {
    RequestMeasurement::start(RequestMeasurementContext {
        connection_id: "codex-oauth",
        canonical_provider_id: "openai",
        api_format: UsageApiFormat::Responses,
        model: "gpt-5.6-sol",
        session_id: Some("session-1"),
        request_id: "request-1",
        turn: Some(1),
        attempt: 1,
        workload: UsageWorkload::Primary,
        fast_mode: FastModeRequest::Fast,
    })
    .unwrap()
}

async fn sse_response(
    body: &'static str,
    keep_open: Duration,
) -> (reqwest::Response, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0_u8; 1024];
        let _ = socket.read(&mut request).await;
        let headers = if keep_open.is_zero() {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n",
                body.len()
            )
        } else {
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\n".to_string()
        };
        socket.write_all(headers.as_bytes()).await.unwrap();
        socket.write_all(body.as_bytes()).await.unwrap();
        tokio::time::sleep(keep_open).await;
    });
    let client = AuthenticatedClient::new_loopback(Duration::from_secs(2)).unwrap();
    let response = client
        .send(client.get(format!("http://{address}/stream")))
        .await
        .unwrap();
    (response, server)
}

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

#[tokio::test]
async fn completed_sse_returns_the_accumulated_result() {
    let body = "data: {\"type\":\"response.output_text.delta\",\"delta\":\"ok\"}\n\ndata: {\"type\":\"response.completed\",\"response\":{\"service_tier\":\"priority\"}}\n\n";
    let (response, server) = sse_response(body, Duration::ZERO).await;
    let mut request_measurement = request_measurement();
    request_measurement.mark_headers();
    let outcome = {
        let mut measurement = StreamMeasurement::new(Some(&mut request_measurement));
        consume_sse_with_timeout(
            &NoopSink,
            response,
            CancellationToken::new(),
            false,
            None,
            "openai",
            "gpt-5.6-sol",
            &[],
            None,
            Duration::from_secs(1),
            &mut measurement,
        )
        .await
        .unwrap()
    };

    assert_eq!(outcome.into_result().content, "ok");
    assert!(request_measurement.timing().first_event_ms.is_some());
    assert!(request_measurement.timing().first_useful_ms.is_some());
    assert_eq!(
        request_measurement.fast_observation().1,
        crate::services::provider_usage::ServiceTierServed::Fast
    );
    server.await.unwrap();
}

#[tokio::test]
async fn closed_or_done_before_completion_is_rejected() {
    for body in ["", "data: [DONE]\n\n"] {
        let (response, server) = sse_response(body, Duration::ZERO).await;
        let mut measurement = StreamMeasurement::new(None);
        let error = consume_sse_with_timeout(
            &NoopSink,
            response,
            CancellationToken::new(),
            false,
            None,
            "openai",
            "gpt-5.6-sol",
            &[],
            None,
            Duration::from_secs(1),
            &mut measurement,
        )
        .await
        .unwrap_err();

        assert_eq!(error, "provider_connection_failed");
        server.await.unwrap();
    }
}

#[tokio::test]
async fn stalled_sse_is_cancelled_by_user_or_idle_deadline() {
    let (response, server) = sse_response("", Duration::from_secs(2)).await;
    let cancel = CancellationToken::new();
    cancel.cancel();
    let mut measurement = StreamMeasurement::new(None);
    let cancelled = consume_sse_with_timeout(
        &NoopSink,
        response,
        cancel,
        false,
        None,
        "openai",
        "gpt-5.6-sol",
        &[],
        None,
        Duration::from_secs(1),
        &mut measurement,
    )
    .await;
    assert_eq!(cancelled.unwrap_err(), "Annulé");
    server.abort();

    let (response, server) = sse_response("", Duration::from_secs(2)).await;
    let mut measurement = StreamMeasurement::new(None);
    let timed_out = consume_sse_with_timeout(
        &NoopSink,
        response,
        CancellationToken::new(),
        false,
        None,
        "openai",
        "gpt-5.6-sol",
        &[],
        None,
        Duration::from_millis(20),
        &mut measurement,
    )
    .await;
    assert_eq!(timed_out.unwrap_err(), "provider_temporarily_unavailable");
    server.abort();
}

#[tokio::test]
async fn oversized_incomplete_sse_is_rejected_before_the_idle_deadline() {
    let (response, server) = oversized_incomplete_sse_response().await;
    let mut measurement = StreamMeasurement::new(None);
    let error = consume_sse_with_timeout(
        &NoopSink,
        response,
        CancellationToken::new(),
        false,
        None,
        "openai",
        "gpt-5.6-sol",
        &[],
        None,
        Duration::from_secs(5),
        &mut measurement,
    )
    .await
    .unwrap_err();

    assert_eq!(error, "provider_connection_failed");
    server.abort();
}

#[test]
fn completed_outcome_variant_remains_distinct_from_compression() {
    let outcome = StreamOutcome::Completed(Default::default());
    assert!(!outcome.is_interrupted());
}
