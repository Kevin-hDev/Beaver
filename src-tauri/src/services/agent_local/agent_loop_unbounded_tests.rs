use super::agent_loop_test_provider;
use super::context_usage_buckets::ContextUsageSeed;
use super::extension_tool_set::ExtensionToolSet;
use super::stream_events::AgentEventEmitter;
use super::types_ollama::{ChatMessage, OllamaThink, StreamResult};
use crate::services::reasoning_fixture_run::FixtureRunContext;
use serde_json::json;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn ollama_loop_continues_past_200_turns() {
    let mut fixture = FixtureRunContext::start().await.expect("fixture");
    let working_dir = fixture.root_for_test();
    let tools = ExtensionToolSet::passthrough(fixture.definitions().into());
    let session =
        super::session_store::create_full("Ollama unbounded", "fixture", "ollama", false, None)
            .await
            .expect("session");
    let session_id = session.id.clone();
    let request_id = uuid::Uuid::new_v4().to_string();
    let script = agent_loop_test_provider::install(&request_id, tool_turns(201));
    let mut messages = vec![ChatMessage::user("continue".into())];
    let cancel = CancellationToken::new();

    let result = super::agent_loop::run_agent_loop(
        &AgentEventEmitter::test(session_id.clone()),
        &mut messages,
        "fixture",
        tools,
        OllamaThink::Bool(false),
        working_dir,
        session_id,
        request_id,
        None,
        cancel.clone(),
        1_000_000,
        1_000_000,
        "auto",
        false,
        ContextUsageSeed::default(),
        false,
        None,
        None,
        Some(&mut fixture),
        None,
    )
    .await;
    let error = match result {
        Ok(_) => panic!("script exhaustion must cancel the loop"),
        Err(error) => error,
    };
    super::session_store::delete_one(&session.id)
        .await
        .expect("delete session");

    assert_eq!(error, "Annulé");
    assert!(cancel.is_cancelled());
    assert_eq!(script.calls(), 201);
    assert_eq!(
        fixture
            .dispatch("fixture.read_note", &json!({}))
            .await
            .unwrap(),
        json!({ "value": "200" })
    );
}

fn tool_turns(count: usize) -> Vec<StreamResult> {
    (0..count)
        .map(|turn| StreamResult {
            tool_calls: vec![(
                "fixture.write_note".into(),
                json!({ "value": turn.to_string() }),
            )],
            tool_call_ids: vec![format!("call-{turn}")],
            ..Default::default()
        })
        .collect()
}
