use crate::services::provider_usage::{UsageApiFormat, UsageContext};
use serde_json::json;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn context() -> UsageContext<'static> {
    UsageContext {
        canonical_provider_id: "anthropic",
        model: "claude-haiku-4-5-20251001",
        api_format: UsageApiFormat::AnthropicMessages,
    }
}

#[test]
fn semantic_stream_reconstructs_blocks_once_and_waits_for_block_stop() {
    let result = super::stream::consume_fixture(
        include_str!("../../../../test-fixtures/anthropic/message-tools-stream.sse"),
        context(),
    )
    .unwrap();

    assert_eq!(result.content, "réponse");
    assert_eq!(result.tool_call_ids, ["toolu_1", "toolu_2"]);
    assert_eq!(result.tool_calls[0].1, json!({"path": "README.md"}));
    assert_eq!(result.usage.as_ref().unwrap().input_tokens, Some(120));
    assert_eq!(result.usage.as_ref().unwrap().output_tokens, Some(30));
    assert_eq!(result.usage.as_ref().unwrap().cached_input_tokens, Some(80));
    assert_eq!(
        result.usage.as_ref().unwrap().cache_write_input_tokens,
        Some(20)
    );
    assert_eq!(result.finish_reason.as_deref(), Some("tool_use"));
}

#[test]
fn omitted_thinking_keeps_exact_signature_without_display_text() {
    let result = super::stream::consume_fixture(
        include_str!("../../../../test-fixtures/anthropic/message-thinking-omitted-stream.sse"),
        context(),
    )
    .unwrap();

    assert_eq!(result.continuation_blocks[0]["thinking"], "");
    assert_eq!(result.continuation_blocks[0]["signature"], "AAE+/==");
}

#[test]
fn a_late_signature_is_created_when_the_start_block_omits_the_field() {
    let fixture = concat!(
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"thinking\",\"thinking\":\"\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"opaque\"}}\n\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"signature_delta\",\"signature\":\"AAE+/==\"}}\n\n",
        "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
        "data: {\"type\":\"message_stop\"}\n\n",
    );

    let result = super::stream::consume_fixture(fixture, context()).unwrap();

    assert_eq!(result.continuation_blocks[0]["thinking"], "opaque");
    assert_eq!(result.continuation_blocks[0]["signature"], "AAE+/==");
}

#[test]
fn refusal_finishes_without_retry_or_tool_call() {
    let result = super::stream::consume_fixture(
        include_str!("../../../../test-fixtures/anthropic/message-refusal-stream.sse"),
        context(),
    )
    .unwrap();

    assert_eq!(result.finish_reason.as_deref(), Some("refusal"));
    assert!(result.tool_calls.is_empty());
}

#[test]
fn provider_error_and_incomplete_arguments_fail_closed() {
    let provider_error = super::stream::consume_fixture(
        include_str!("../../../../test-fixtures/anthropic/message-error-stream.sse"),
        context(),
    );
    assert_eq!(provider_error.unwrap_err(), "provider_request_rejected");

    let incomplete = "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"read_file\",\"input\":{}}}\n\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"path\\\":\"}}\n\ndata: {\"type\":\"content_block_stop\",\"index\":0}\n";
    assert_eq!(
        super::stream::consume_fixture(incomplete, context()).unwrap_err(),
        "provider_stream_invalid"
    );
}

#[test]
fn unknown_event_is_ignored_but_oversized_usage_is_rejected() {
    let unknown =
        "data: {\"type\":\"future_event\",\"payload\":true}\n\ndata: {\"type\":\"message_stop\"}\n";
    assert!(super::stream::consume_fixture(unknown, context()).is_ok());

    let oversized = "data: {\"type\":\"message_start\",\"message\":{\"usage\":{\"input_tokens\":10000000001}}}\n";
    assert_eq!(
        super::stream::consume_fixture(oversized, context()).unwrap_err(),
        "provider_stream_invalid"
    );
}

async fn response(body: &str, request_id: Option<&str>) -> reqwest::Response {
    let server = MockServer::start().await;
    let mut template = ResponseTemplate::new(200)
        .insert_header("content-type", "text/event-stream")
        .set_body_string(body);
    if let Some(request_id) = request_id {
        template = template.insert_header("request-id", request_id);
    }
    Mock::given(any())
        .respond_with(template)
        .mount(&server)
        .await;
    let client =
        crate::services::secure_http::AuthenticatedClient::new_loopback(Duration::from_secs(2))
            .unwrap();
    client.send(client.get(server.uri())).await.unwrap()
}

#[tokio::test]
async fn network_consumer_honors_cancellation_before_reading_events() {
    let response = response("data: {\"type\":\"ping\"}\n\n", None).await;
    let cancel = CancellationToken::new();
    cancel.cancel();

    let error = super::stream::consume_stream(
        &crate::services::agent_local::stream_events::AgentEventEmitter::test("session".into()),
        response,
        cancel,
        true,
        None,
        &[],
        context(),
        None,
        None,
    )
    .await
    .unwrap_err();

    assert_eq!(error, "Annulé");
}

#[tokio::test]
async fn network_consumer_observes_safe_request_id_and_finish_reason() {
    use crate::services::llm::fast_mode::FastModeRequest;
    use crate::services::provider_usage::{
        RequestMeasurement, RequestMeasurementContext, UsageWorkload,
    };

    let response = response(
        include_str!("../../../../test-fixtures/anthropic/message-refusal-stream.sse"),
        Some("req_123"),
    )
    .await;
    let mut measurement = RequestMeasurement::start(RequestMeasurementContext {
        connection_id: "openai",
        canonical_provider_id: "openai",
        api_format: UsageApiFormat::ChatCompletions,
        model: "fixture",
        session_id: Some("session-1"),
        request_id: "request-1",
        turn: Some(1),
        attempt: 1,
        workload: UsageWorkload::Primary,
        fast_mode: FastModeRequest::Standard,
    })
    .unwrap();

    super::stream::consume_stream(
        &crate::services::agent_local::stream_events::AgentEventEmitter::test("session".into()),
        response,
        CancellationToken::new(),
        true,
        None,
        &[],
        context(),
        None,
        Some(&mut measurement),
    )
    .await
    .unwrap();

    assert_eq!(
        measurement.provider_metadata(),
        (Some("req_123"), Some("refusal"))
    );
}

#[tokio::test]
async fn network_consumer_persists_exact_completed_anthropic_blocks() {
    use crate::services::agent_local::types_ollama::StreamOutcome;
    use crate::services::llm::reasoning_wire::{ReasoningCapture, ReasoningCaptureContext};
    use crate::services::reasoning_continuity::contract::{
        CredentialScope, ReasoningModeId, RouteId,
    };
    use crate::services::reasoning_continuity::envelope::ContinuationState;

    let response = response(
        include_str!("../../../../test-fixtures/anthropic/message-tools-stream.sse"),
        None,
    )
    .await;
    let capture = ReasoningCapture::new(ReasoningCaptureContext {
        route_id: RouteId::Anthropic,
        model_id: "claude-haiku-4-5-20251001".into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::Low,
    })
    .unwrap();

    let outcome = super::stream::consume_stream(
        &crate::services::agent_local::stream_events::AgentEventEmitter::test("session".into()),
        response,
        CancellationToken::new(),
        true,
        None,
        &[],
        context(),
        Some(capture),
        None,
    )
    .await
    .unwrap();
    let StreamOutcome::Completed(result) = outcome else {
        panic!("completed Anthropic stream")
    };
    let envelope = result.continuation.expect("signed continuation");
    let ContinuationState::AnthropicBlocks { blocks } = envelope.continuation else {
        panic!("Anthropic blocks")
    };

    assert_eq!(blocks[0]["type"], "thinking");
    assert!(blocks[0]["signature"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert_eq!(blocks[2]["id"], "toolu_1");
    assert_eq!(envelope.tool_links[0].provider_call_id, "toolu_1");
}

#[tokio::test]
async fn network_consumer_exposes_anthropic_thinking_to_the_session() {
    use crate::services::agent_local::types_ollama::StreamOutcome;

    let response = response(
        include_str!("../../../../test-fixtures/anthropic/message-tools-stream.sse"),
        None,
    )
    .await;
    let outcome = super::stream::consume_stream(
        &crate::services::agent_local::stream_events::AgentEventEmitter::test("session".into()),
        response,
        CancellationToken::new(),
        true,
        None,
        &[],
        context(),
        None,
        None,
    )
    .await
    .unwrap();
    let StreamOutcome::Completed(result) = outcome else {
        panic!("completed Anthropic stream")
    };

    assert_eq!(result.thinking, "private");
}

#[tokio::test]
async fn network_consumer_interrupts_for_compression_without_persisting_partial_blocks() {
    use crate::services::agent_local::types_ollama::StreamOutcome;
    let body = format!(
        "data: {{\"type\":\"content_block_start\",\"index\":0,\"content_block\":{{\"type\":\"text\",\"text\":\"\"}}}}\n\ndata: {{\"type\":\"content_block_delta\",\"index\":0,\"delta\":{{\"type\":\"text_delta\",\"text\":\"{}\"}}}}\n\n",
        "x".repeat(200)
    );
    let response = response(&body, None).await;
    let budget =
        crate::services::compress::realtime_budget::RealtimeBudget::new(true, 100, 1, 0).unwrap();

    let outcome = super::stream::consume_stream(
        &crate::services::agent_local::stream_events::AgentEventEmitter::test("session".into()),
        response,
        CancellationToken::new(),
        true,
        Some(budget),
        &[],
        context(),
        None,
        None,
    )
    .await
    .unwrap();
    let StreamOutcome::InterruptedForCompression(result) = outcome else {
        panic!("compression interruption")
    };
    assert_eq!(result.content.len(), 200);
    assert!(result.continuation.is_none());
}
