use std::time::Duration;

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
        None,
    )
    .await;

    assert_eq!(result.unwrap_err(), "provider_connection_failed");
}

#[tokio::test]
async fn explicit_done_completes_the_stream() {
    let (_server, response) = streaming_response(concat!(
        "data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n",
        "data: [DONE]\n\n",
    ))
    .await;

    let result = consume_silent(
        response,
        CancellationToken::new(),
        Duration::from_secs(2),
        crate::services::provider_usage::UsageContext::chat("openai", "gpt-5.6-sol"),
        None,
    )
    .await
    .unwrap();

    assert_eq!(result.content, "ok");
}
