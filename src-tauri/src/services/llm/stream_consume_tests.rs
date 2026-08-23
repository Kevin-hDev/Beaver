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
        Some(&mut measurement),
    )
    .await
    .unwrap();

    assert_eq!(
        measurement.fast_observation().1,
        crate::services::provider_usage::ServiceTierServed::Fast
    );
}
