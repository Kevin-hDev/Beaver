use super::openrouter;
use serde_json::{json, Value};

#[test]
fn routing_projection_keeps_evidence_not_prompts_or_free_form_details() {
    let source = json!({
        "openrouter_metadata": {
            "attempt": 1, "strategy": "direct", "summary": "private-summary",
            "endpoints": {"total": 2, "available": [
                {"provider":"DeepInfra", "model":"nvidia/test:free", "selected":false},
                {"provider":"CoreWeave", "selected":false}
            ]},
            "attempts": [{"provider":"DeepInfra", "status":429}],
            "pipeline": [{"type":"guardrail", "name":"content-filter",
                "summary":"private-summary", "data":{"action":"blocked", "flagged":true,
                "patterns":["private-prompt"]}}]
        }, "error":{"message":"private-error"}, "choices":[{"text":"private-answer"}]
    });
    let safe = openrouter::project(&source).unwrap();
    let safe = serde_json::to_value(safe).unwrap();
    assert_eq!(safe["attempt"], 1);
    assert_eq!(safe["endpoints"][0]["provider"], "DeepInfra");
    assert_eq!(safe["attempts"][0]["status"], 429);
    assert_eq!(safe["pipeline"][0]["action"], "blocked");
    assert_eq!(safe["pipeline"][0]["flagged"], true);
    assert!(!safe.to_string().contains("private-"));
}

#[test]
fn absent_or_malformed_metadata_is_not_an_error_or_an_invented_cause() {
    for source in [
        json!({}),
        json!({"openrouter_metadata":null}),
        json!({"openrouter_metadata":"invalid"}),
    ] {
        assert!(openrouter::project(&source).is_none());
    }
    let safe = serde_json::to_value(
        openrouter::project(&json!({
            "openrouter_metadata":{"attempt":-1,"endpoints":{"total":"bad"},
            "attempts":[{"status":999999}],"unknown":"ignored"}
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(safe["attempt"].is_null());
    assert!(safe["attempts"][0]["status"].is_null());
    assert!(!safe.to_string().contains("ignored"));
}

#[test]
fn external_routing_collections_and_labels_are_bounded_and_redacted() {
    let item = json!({"provider":"sk-test-sentinel-123456789", "model":"a".repeat(200),
        "name":"Bearer private-token", "type":"guardrail"});
    let safe = serde_json::to_value(
        openrouter::project(&json!({
            "openrouter_metadata": {
                "endpoints":{"available": vec![item.clone();100]},
                "attempts":vec![item.clone();100], "pipeline":vec![item;100]
            }
        }))
        .unwrap(),
    )
    .unwrap();
    for field in ["endpoints", "attempts", "pipeline"] {
        assert!(safe[field].as_array().unwrap().len() <= 16);
    }
    assert_eq!(safe["truncated"], true);
    let serialized = safe.to_string();
    assert!(!serialized.contains("sentinel"));
    assert!(!serialized.contains("private-token"));
    assert!(!serialized.contains(&"a".repeat(200)));
}

#[tokio::test]
async fn real_sse_readers_persist_final_routing_metadata_even_on_stream_error() {
    use crate::services::agent_local::stream_events::AgentEventEmitter;
    use crate::services::llm::{route_profile, stream_consume, stream_silent_consume};
    use crate::services::provider_usage::UsageContext;
    use tokio_util::sync::CancellationToken;
    for silent in [false, true] {
        for fails in [false, true] {
            let id = uuid::Uuid::new_v4().to_string();
            let last = if fails {
                json!({"error":{"code":429,"message":"private-error"},
                    "openrouter_metadata":{"attempt":1,"attempts":[{"provider":"DeepInfra","status":429}]}})
            } else {
                json!({"choices":[],"openrouter_metadata":{"attempt":1,"attempts":[{"provider":"DeepInfra","status":200}]}})
            };
            let body = format!("data: {{\"id\":\"gen-fixture123\",\"choices\":[{{\"delta\":{{\"content\":\"OK\"}}}}]}}\n\ndata: {last}\n\ndata: [DONE]\n\n");
            let mut response = response(200, &body).await;
            openrouter::attach(&mut response, "openrouter", "nvidia/test", Some(&id));
            let result = if silent {
                stream_silent_consume::consume_silent(
                    response,
                    CancellationToken::new(),
                    std::time::Duration::from_secs(2),
                    UsageContext::chat("openrouter", "nvidia/test"),
                    route_profile::FragmentMode::DifferentialFragments,
                    route_profile::ErrorPolicy::OpenAiCompatible,
                    None,
                )
                .await
            } else {
                stream_consume::consume_stream(
                    &AgentEventEmitter::test(id.clone()),
                    response,
                    CancellationToken::new(),
                    true,
                    None,
                    &[],
                    UsageContext::chat("openrouter", "nvidia/test"),
                    route_profile::FragmentMode::DifferentialFragments,
                    route_profile::ErrorPolicy::OpenAiCompatible,
                    None,
                    None,
                )
                .await
                .map(|outcome| outcome.into_result())
            };
            assert_eq!(result.is_err(), fails);
            if !fails {
                assert_eq!(result.unwrap().content, "OK");
            }
            let entries = entries(&id);
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0]["generation_id"], "gen-fixture123");
            assert_eq!(
                entries[0]["routing"]["attempts"][0]["status"],
                if fails { 429 } else { 200 }
            );
            assert!(!entries[0].to_string().contains("private-error"));
        }
    }
}

async fn response(status: u16, body: &str) -> reqwest::Response {
    use wiremock::{Mock, MockServer, ResponseTemplate};
    let server = MockServer::start().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(
            ResponseTemplate::new(status)
                .set_body_string(body)
                .insert_header("x-generation-id", "gen-header123"),
        )
        .mount(&server)
        .await;
    let client = crate::services::secure_http::AuthenticatedClient::new_loopback(
        std::time::Duration::from_secs(2),
    )
    .unwrap();
    client.send(client.get(server.uri())).await.unwrap()
}

#[tokio::test]
async fn http_error_reader_preserves_header_id_and_sanitized_routing_without_changing_body() {
    for body in [
        r#"{"error":{"code":403,"message":"private-refusal"},"openrouter_metadata":{"attempt":0}}"#,
        r#"{"error":{"code":429,"message":"private-refusal"}}"#,
        "invalid-json",
    ] {
        let id = uuid::Uuid::new_v4().to_string();
        let mut response = response(403, body).await;
        openrouter::attach(&mut response, "openrouter", "meta/test", Some(&id));
        let result = crate::services::llm::stream_http::read_provider_error(response).await;
        assert_eq!(result.as_str(), body);
        let entries = entries(&id);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["http_status"], 403);
        assert_eq!(entries[0]["generation_id"], "gen-header123");
        assert!(!entries[0].to_string().contains("private-refusal"));
        if body.contains("openrouter_metadata") {
            assert_eq!(entries[0]["routing"]["attempt"], 0);
        } else {
            assert!(entries[0]["routing"].is_null());
        }
    }
}

#[tokio::test]
async fn unrelated_provider_responses_never_create_openrouter_diagnostics() {
    let id = uuid::Uuid::new_v4().to_string();
    let mut response = response(200, "unused").await;
    openrouter::attach(&mut response, "google", "gemini-test", Some(&id));
    assert!(openrouter::take(&mut response).is_none());
    assert!(entries(&id).is_empty());
}

#[tokio::test]
async fn repeated_metadata_is_bounded_and_writes_only_one_observation() {
    let id = uuid::Uuid::new_v4().to_string();
    let mut response = response(200, "unused").await;
    response.headers_mut().insert(
        "x-generation-id",
        "gen-sk-test-sentinel-123456789".parse().unwrap(),
    );
    openrouter::attach(&mut response, "openrouter", "meta/test", Some(&id));
    let mut observation = openrouter::take(&mut response).unwrap();
    for index in 0..100 {
        observation.observe(&json!({"openrouter_metadata":{"attempt":index}}));
    }
    drop(observation);
    let entries = entries(&id);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["snapshots_truncated"], true);
    assert!(entries[0]["routing"]["attempt"].as_u64().unwrap() < 16);
    assert!(!entries[0].to_string().contains("sentinel"));
    assert!(entries[0].to_string().len() < 32 * 1024);
}

#[tokio::test]
async fn cancellation_flushes_header_evidence_without_claiming_a_completed_generation() {
    let id = uuid::Uuid::new_v4().to_string();
    let mut response = response(200, "").await;
    openrouter::attach(&mut response, "openrouter", "meta/test", Some(&id));
    let cancel = tokio_util::sync::CancellationToken::new();
    cancel.cancel();
    let result = crate::services::llm::stream_silent_consume::consume_silent(
        response,
        cancel,
        std::time::Duration::from_secs(2),
        crate::services::provider_usage::UsageContext::chat("openrouter", "meta/test"),
        crate::services::llm::route_profile::FragmentMode::DifferentialFragments,
        crate::services::llm::route_profile::ErrorPolicy::OpenAiCompatible,
        None,
    )
    .await;
    assert!(result.is_err());
    let entries = entries(&id);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["http_status"], 200);
    assert!(entries[0]["routing"].is_null());
}

fn entries(id: &str) -> Vec<Value> {
    std::fs::read_to_string(super::log_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|entry| entry["request_id"] == id && entry["transport"] == "openrouter_routing")
        .collect()
}
