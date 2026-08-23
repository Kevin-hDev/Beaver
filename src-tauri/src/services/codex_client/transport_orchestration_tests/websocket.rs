use super::{assert_http_capture, assert_websocket_capture, messages};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::codex_client::test_transport::{
    CodexTransportScenario, HttpReply, WebSocketReply,
};
use crate::services::codex_client::{stream, stream_measurement::StreamMeasurement, websocket};
use crate::services::llm::fast_mode::FastModeRequest;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn request_keeps_payload_and_handshake_aligned_for_every_mode() {
    for (mode, model, expected_tier, expected_hint) in [
        (
            FastModeRequest::Fast,
            "gpt-5.6-sol",
            Some("priority"),
            "model=gpt-5.6-sol;tier=priority",
        ),
        (
            FastModeRequest::Standard,
            "gpt-5.6-sol",
            None,
            "model=gpt-5.6-sol",
        ),
        (
            FastModeRequest::Unsupported,
            "gpt-5.4-mini",
            None,
            "model=gpt-5.4-mini",
        ),
    ] {
        let scenario = CodexTransportScenario::start(None, Some(WebSocketReply::Success)).await;
        let emitter = AgentEventEmitter::test("session-ws-routing".to_string());
        let mut measurement = StreamMeasurement::new(None);
        websocket::stream_chat(
            &emitter,
            "session-ws-routing",
            model,
            &messages(),
            &[],
            None,
            mode,
            CancellationToken::new(),
            false,
            None,
            &mut measurement,
        )
        .await
        .expect("loopback WebSocket succeeds");
        scenario.wait_for_websocket_captures(1).await;

        let captures = scenario.websocket_captures();
        assert_eq!(captures.len(), 1);
        assert_websocket_capture(&captures[0], model, expected_tier, expected_hint);
    }
}

#[tokio::test]
async fn fallback_to_http_keeps_the_original_fast_capture() {
    let session = crate::services::agent_local::session_store::create_with_project_and_fast_mode(
        "Codex fallback capture",
        "gpt-5.6-sol",
        "codex-oauth",
        None,
        false,
    )
    .await
    .expect("create Standard session");
    let scenario = CodexTransportScenario::start(
        Some(vec![HttpReply::Success]),
        Some(WebSocketReply::Unavailable),
    )
    .await;
    let emitter = AgentEventEmitter::test(session.id.clone());
    let result = stream::stream_chat_with_budget(
        &emitter,
        &session.id,
        "request-fallback-fast",
        "gpt-5.6-sol",
        &messages(),
        &[],
        None,
        FastModeRequest::Fast,
        CancellationToken::new(),
        false,
        None,
        None,
    )
    .await;
    scenario.wait_for_websocket_captures(1).await;
    let websocket_captures = scenario.websocket_captures();
    let http_captures = scenario.http_captures();
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .expect("delete session");

    result.expect("HTTP fallback succeeds");
    assert_eq!(websocket_captures.len(), 1);
    assert_eq!(http_captures.len(), 1);
    assert_websocket_capture(
        &websocket_captures[0],
        "gpt-5.6-sol",
        Some("priority"),
        "model=gpt-5.6-sol;tier=priority",
    );
    assert_http_capture(
        &http_captures[0],
        "gpt-5.6-sol",
        Some("priority"),
        "model=gpt-5.6-sol;tier=priority",
    );
}
