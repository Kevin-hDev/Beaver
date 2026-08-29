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
        crate::services::provider_usage::UsageContext::chat("openai", "fixture"),
        mode,
        crate::services::llm::route_profile::ErrorPolicy::Responses,
        None,
        None,
    )
    .await
    .unwrap()
    .into_result()
    .content
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
