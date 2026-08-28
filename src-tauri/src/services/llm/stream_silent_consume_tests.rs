use std::time::Duration;

use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::provider_usage::{
    RequestMeasurement, RequestMeasurementContext, UsageApiFormat, UsageWorkload,
};
use tokio_util::sync::CancellationToken;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;

async fn streaming_response(body: &str) -> (MockServer, reqwest::Response) {
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
            .expect("client");
    let response = client
        .send(client.get(server.uri()))
        .await
        .expect("response");
    (server, response)
}

#[tokio::test]
async fn eof_before_done_is_rejected_as_truncated() {
    let (_server, response) =
        streaming_response("data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n").await;

    let result = consume_silent(
        response,
        CancellationToken::new(),
        Duration::from_secs(2),
        crate::services::provider_usage::UsageContext::chat("openai", "gpt-5.6-sol"),
        crate::services::llm::route_profile::FragmentMode::DifferentialFragments,
        crate::services::llm::route_profile::ErrorPolicy::Responses,
        None,
    )
    .await;

    assert_eq!(result.unwrap_err(), "provider_connection_failed");
}

#[tokio::test]
async fn explicit_done_completes_the_stream() {
    let (_server, response) = streaming_response(concat!(
        "data: {\"service_tier\":\"fast\",\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n",
        "data: [DONE]\n\n",
    ))
    .await;
    let mut request_measurement = RequestMeasurement::start(RequestMeasurementContext {
        connection_id: "openai",
        canonical_provider_id: "openai",
        api_format: UsageApiFormat::ChatCompletions,
        model: "gpt-5.6-sol",
        session_id: Some("session-1"),
        request_id: "request-1",
        turn: Some(1),
        attempt: 1,
        workload: UsageWorkload::Compression,
        fast_mode: FastModeRequest::Fast,
    })
    .unwrap();

    let result = consume_silent(
        response,
        CancellationToken::new(),
        Duration::from_secs(2),
        crate::services::provider_usage::UsageContext::chat("openai", "gpt-5.6-sol"),
        crate::services::llm::route_profile::FragmentMode::DifferentialFragments,
        crate::services::llm::route_profile::ErrorPolicy::Responses,
        Some(&mut request_measurement),
    )
    .await
    .unwrap();

    assert_eq!(result.content, "ok");
    assert_eq!(
        request_measurement.fast_observation().1,
        crate::services::provider_usage::ServiceTierServed::Fast
    );
}

#[tokio::test]
async fn embedded_provider_error_is_not_treated_as_a_valid_summary() {
    let (_server, response) = streaming_response(concat!(
        "data: {\"error\":{\"code\":503,\"message\":\"private\"}}\n\n",
        "data: [DONE]\n\n",
    ))
    .await;

    let error = consume_silent(
        response,
        CancellationToken::new(),
        Duration::from_secs(2),
        crate::services::provider_usage::UsageContext::chat("openai", "fixture"),
        crate::services::llm::route_profile::FragmentMode::DifferentialFragments,
        crate::services::llm::route_profile::ErrorPolicy::Responses,
        None,
    )
    .await
    .unwrap_err();

    assert_eq!(error, "provider_temporarily_unavailable");
    assert!(!error.contains("private"));
}
