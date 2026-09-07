use super::build_request;
use crate::services::reasoning_fixture_budget::{run_scoped, FixtureLimits};
use crate::services::agent_local::types_ollama::OllamaThink;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn fixture_scope_adds_bounded_native_output_without_changing_normal_chat() {
    let normal = build_request("fixture", &[], &[], OllamaThink::Bool(false));
    assert!(normal.options.is_none());
    let limits = FixtureLimits::from_values(Some("7"), None, None).unwrap();
    run_scoped(limits, CancellationToken::new(), async {
        let request = build_request("fixture", &[], &[], OllamaThink::Bool(false));
        let wire = super::super::ollama_wire::chat_request(&request, &request.messages).unwrap();
        assert_eq!(wire["options"]["num_predict"], 7);
        Ok::<(), String>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn fixture_scope_native_sender_refuses_an_extra_attempt() {
    use super::super::ollama_client::OllamaClient;
    use super::super::ollama_stream_request::{open_chat_response, ReplayDiagnosticContext, RetryCounts};
    use super::super::stream_events::AgentEventEmitter;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(ResponseTemplate::new(200)).mount(&server).await;
    let ollama = OllamaClient::with_base_url(&server.uri()).unwrap();
    let emitter = AgentEventEmitter::test("fixture-native-budget".into());
    let limits = FixtureLimits::from_values(Some("7"), Some("1"), None).unwrap();
    run_scoped(limits, CancellationToken::new(), async {
        let request = build_request("fixture", &[], &[], OllamaThink::Bool(false));
        for permitted in [true, false] {
            let result = open_chat_response(
                &ollama, &emitter, &request, &CancellationToken::new(),
                RetryCounts { parser_retries: 0, server_retries: 0 }, false,
                ReplayDiagnosticContext { session_id: "fixture-native-budget", request_id: "fixture-native-budget" },
            ).await;
            assert_eq!(result.is_ok(), permitted);
        }
        Ok(())
    }).await.unwrap();
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].body_json::<serde_json::Value>().unwrap()["options"]["num_predict"], 7);
}
