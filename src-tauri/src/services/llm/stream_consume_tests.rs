use std::time::Duration;

use tokio_util::sync::CancellationToken;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::provider_usage::{
    RequestMeasurement, RequestMeasurementContext, UsageApiFormat, UsageWorkload,
};

async fn consume_text_fixture(
    body: &str,
    mode: crate::services::llm::route_profile::FragmentMode,
) -> String {
    consume_fixture(body, mode, "openai").await.content
}

async fn consume_fixture(
    body: &str,
    mode: crate::services::llm::route_profile::FragmentMode,
    provider: &str,
) -> crate::services::agent_local::types_ollama::StreamResult {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;
    let client =
        crate::services::secure_http::AuthenticatedClient::new_loopback(Duration::from_secs(2))
            .unwrap();
    let response = client.send(client.get(server.uri())).await.unwrap();

    consume_stream(
        &AgentEventEmitter::test("fragment-fixture".into()),
        response,
        CancellationToken::new(),
        true,
        None,
        &[],
        crate::services::provider_usage::UsageContext::chat(provider, "fixture"),
        mode,
        crate::services::llm::route_profile::ErrorPolicy::Responses,
        None,
        None,
    )
    .await
    .unwrap()
    .into_result()
}

#[tokio::test]
async fn chat_sse_preserves_token_limit_finish_reason_and_billed_usage() {
    let result = consume_fixture(
        concat!(
            "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"length\"}]}\n\n",
            "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"length\"}],\"usage\":{\"prompt_tokens\":739,\"completion_tokens\":512,\"total_tokens\":1251,\"completion_tokens_details\":{\"reasoning_tokens\":505}}}\n\n",
            "data: [DONE]\n\n",
        ),
        crate::services::llm::route_profile::FragmentMode::DifferentialFragments,
        "openrouter",
    ).await;
    assert_eq!(result.done_reason.as_deref(), Some("length"));
    assert!(result.content.is_empty());
    assert_eq!(result.completion_error, Some("provider_output_limit"));
    let usage = result.usage.unwrap();
    assert_eq!(usage.output_tokens, Some(512));
    assert_eq!(usage.reasoning_output_tokens, Some(505));
}

#[tokio::test]
async fn terminal_limits_discard_tools_even_when_the_arguments_are_valid_json() {
    for reason in ["length", "content_filter", "tool_calls"] {
        let body = format!(
            "data: {{\"choices\":[{{\"delta\":{{\"tool_calls\":[{{\"index\":0,\"id\":\"call_1\",\"type\":\"function\",\"function\":{{\"name\":\"write_note\",\"arguments\":\"{{}}\"}}}}]}},\"finish_reason\":\"{reason}\"}}]}}\n\ndata: [DONE]\n\n"
        );
        let result = consume_fixture(
            &body,
            crate::services::llm::route_profile::FragmentMode::DifferentialFragments,
            "openrouter",
        )
        .await;
        if reason == "tool_calls" {
            assert_eq!(result.tool_calls.len(), 1);
            assert_eq!(result.completion_error, None);
        } else {
            assert!(result.tool_calls.is_empty());
            assert!(result.tool_call_ids.is_empty());
            assert!(result.completion_error.is_some());
        }
    }
}

#[tokio::test]
async fn done_without_answer_or_tool_is_not_a_success() {
    let result = consume_fixture(
        "data: {\"choices\":[{\"delta\":{\"reasoning\":\"thinking only\"},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n",
        crate::services::llm::route_profile::FragmentMode::DifferentialFragments,
        "openrouter",
    ).await;
    assert_eq!(result.thinking, "thinking only");
    assert_eq!(result.completion_error, Some("provider_empty_response"));
}

#[tokio::test]
async fn google_interactive_usage_counts_all_generated_tokens() {
    let result = consume_fixture(
        concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"323\"}}]}\n\n",
            "data: {\"usage\":{\"prompt_tokens\":134,\"completion_tokens\":3,\"total_tokens\":217}}\n\n",
            "data: [DONE]\n\n",
        ),
        crate::services::llm::route_profile::FragmentMode::DifferentialFragments,
        "google",
    ).await;
    assert_eq!(result.content, "323");
    assert_eq!(result.eval_count, Some(83));
    assert_eq!(result.prompt_tokens, Some(134));
    let usage = result.usage.unwrap();
    assert_eq!(usage.output_tokens, Some(83));
    assert_eq!(usage.total_tokens, Some(217));
    assert_eq!(usage.reasoning_output_tokens, None);
}

#[tokio::test]
async fn interactive_sse_reader_preserves_differential_and_cumulative_text() {
    let differential = consume_text_fixture(
        concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Bon\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"jour\"}}]}\n\n",
            "data: [DONE]\n\n",
        ),
        crate::services::llm::route_profile::FragmentMode::DifferentialFragments,
    )
    .await;
    let cumulative = consume_text_fixture(
        concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Bon\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"Bonjour\"}}]}\n\n",
            "data: [DONE]\n\n",
        ),
        crate::services::llm::route_profile::FragmentMode::CumulativeFragments,
    )
    .await;

    assert_eq!(differential, "Bonjour");
    assert_eq!(cumulative, differential);
}

#[tokio::test]
async fn chat_sse_consumer_observes_the_served_tier() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(concat!(
                    "data: {\"service_tier\":\"priority\",\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n",
                    "data: [DONE]\n\n",
                )),
        )
        .mount(&server)
        .await;
    let client =
        crate::services::secure_http::AuthenticatedClient::new_loopback(Duration::from_secs(2))
            .unwrap();
    let response = client.send(client.get(server.uri())).await.unwrap();
    let mut measurement = RequestMeasurement::start(RequestMeasurementContext {
        connection_id: "openai",
        canonical_provider_id: "openai",
        api_format: UsageApiFormat::ChatCompletions,
        model: "gpt-5.6-sol",
        session_id: Some("session-1"),
        request_id: "request-1",
        turn: Some(1),
        attempt: 1,
        workload: UsageWorkload::Primary,
        fast_mode: FastModeRequest::Fast,
    })
    .unwrap();

    consume_stream(
        &AgentEventEmitter::test("session-1".into()),
        response,
        CancellationToken::new(),
        false,
        None,
        &[],
        crate::services::provider_usage::UsageContext::chat("openai", "gpt-5.6-sol"),
        crate::services::llm::route_profile::FragmentMode::DifferentialFragments,
        crate::services::llm::route_profile::ErrorPolicy::Responses,
        None,
        Some(&mut measurement),
    )
    .await
    .unwrap();

    assert_eq!(
        measurement.fast_observation().1,
        crate::services::provider_usage::ServiceTierServed::Fast
    );
}

#[tokio::test]
async fn provider_error_inside_a_successful_http_stream_fails_closed() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(concat!(
                    "data: {\"error\":{\"code\":429,\"message\":\"private\"}}\n\n",
                    "data: [DONE]\n\n",
                )),
        )
        .mount(&server)
        .await;
    let client =
        crate::services::secure_http::AuthenticatedClient::new_loopback(Duration::from_secs(2))
            .unwrap();
    let response = client.send(client.get(server.uri())).await.unwrap();

    let error = consume_stream(
        &AgentEventEmitter::test("session-error".into()),
        response,
        CancellationToken::new(),
        false,
        None,
        &[],
        crate::services::provider_usage::UsageContext::chat("openai", "fixture"),
        crate::services::llm::route_profile::FragmentMode::DifferentialFragments,
        crate::services::llm::route_profile::ErrorPolicy::Responses,
        None,
        None,
    )
    .await
    .unwrap_err();

    assert_eq!(error, "rate_limit");
    assert!(!error.contains("private"));
}

#[tokio::test]
async fn provider_error_discards_an_incomplete_tool_call() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(concat!(
                    "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"name\":\"partial\",\"arguments\":\"{\"}}]}}]}\n\n",
                    "data: {\"error\":{\"code\":500}}\n\n",
                )),
        )
        .mount(&server)
        .await;
    let client =
        crate::services::secure_http::AuthenticatedClient::new_loopback(Duration::from_secs(2))
            .unwrap();
    let response = client.send(client.get(server.uri())).await.unwrap();

    let error = consume_stream(
        &AgentEventEmitter::test("session-partial-tool".into()),
        response,
        CancellationToken::new(),
        false,
        None,
        &[],
        crate::services::provider_usage::UsageContext::chat("openai", "fixture"),
        crate::services::llm::route_profile::FragmentMode::DifferentialFragments,
        crate::services::llm::route_profile::ErrorPolicy::Responses,
        None,
        None,
    )
    .await
    .unwrap_err();

    assert_eq!(error, "provider_temporarily_unavailable");
}
