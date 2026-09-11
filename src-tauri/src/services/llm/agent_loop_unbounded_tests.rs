use super::fast_mode::FastModeRequest;
use crate::services::agent_local::agent_loop_test_provider;
use crate::services::agent_local::context_usage_buckets::ContextUsageSeed;
use crate::services::agent_local::extension_tool_set::ExtensionToolSet;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamResult};
use crate::services::reasoning_fixture_run::FixtureRunContext;
use serde_json::json;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn cloud_loop_continues_past_200_turns() {
    let mut fixture = FixtureRunContext::start().await.expect("fixture");
    let working_dir = fixture.root_for_test();
    let tools = ExtensionToolSet::passthrough(fixture.definitions().into());
    let session = crate::services::agent_local::session_store::create_full(
        "Cloud unbounded",
        "fixture",
        "fixture",
        false,
        None,
    )
    .await
    .expect("session");
    let session_id = session.id.clone();
    let request_id = uuid::Uuid::new_v4().to_string();
    let script = agent_loop_test_provider::install(&request_id, tool_turns(201));
    let mut messages = vec![ChatMessage::user("continue".into())];
    let cancel = CancellationToken::new();

    let result = super::agent_loop::run_agent_loop(
        &AgentEventEmitter::test(session_id.clone()),
        "fixture",
        FastModeRequest::Unsupported,
        "fixture",
        &mut messages,
        tools,
        false,
        None,
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
        None,
        Some(&mut fixture),
        None,
    )
    .await;
    let error = match result {
        Ok(_) => panic!("script exhaustion must cancel the loop"),
        Err(error) => error,
    };
    crate::services::agent_local::session_store::delete_one(&session.id)
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

#[tokio::test]
async fn parent_follows_child_past_200_turns() {
    use crate::services::agent_local::{
        subagent_hidden_reports, subagent_registry, subagent_status,
    };

    let parent = crate::services::agent_local::session_store::create_full(
        "Parent unbounded",
        "fixture",
        "fixture",
        false,
        None,
    )
    .await
    .expect("parent session");
    let child_id = uuid::Uuid::new_v4().to_string();
    subagent_registry::register(&parent.id, &child_id, CancellationToken::new())
        .await
        .expect("register child");

    let request_id = uuid::Uuid::new_v4().to_string();
    let mut responses = tool_turns(200);
    responses.push(StreamResult::default());
    responses.extend(tool_turns(1));
    let script = agent_loop_test_provider::install(&request_id, responses);
    let session_id = parent.id.clone();
    let task = tokio::spawn(async move {
        let mut fixture = FixtureRunContext::start().await.expect("fixture");
        let working_dir = fixture.root_for_test();
        let tools = ExtensionToolSet::passthrough(fixture.definitions().into());
        let mut messages = vec![ChatMessage::user("wait for child".into())];
        super::agent_loop::run_agent_loop(
            &AgentEventEmitter::test(session_id.clone()),
            "fixture",
            FastModeRequest::Unsupported,
            "fixture",
            &mut messages,
            tools,
            false,
            None,
            working_dir,
            session_id,
            request_id,
            None,
            CancellationToken::new(),
            1_000_000,
            1_000_000,
            "auto",
            false,
            ContextUsageSeed::default(),
            None,
            Some(&mut fixture),
            None,
        )
        .await
    });

    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while script.calls() < 201 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("reach no-tool turn 201");
    subagent_hidden_reports::append(
        &parent.id,
        subagent_hidden_reports::build_report(
            child_id.clone(),
            "Child".into(),
            "explorer".into(),
            subagent_status::COMPLETED.into(),
            "Child report".into(),
        ),
    )
    .await
    .expect("persist child report");
    subagent_registry::complete_child(
        &child_id,
        subagent_registry::SubagentTerminalKind::ReportPersisted,
    )
    .await
    .expect("complete child");

    let result = task.await.expect("loop task");
    let error = match result {
        Ok(_) => panic!("script exhaustion must cancel the loop"),
        Err(error) => error,
    };
    assert_eq!(error, "Annulé");
    assert_eq!(script.calls(), 202);

    subagent_registry::unregister(&child_id).await;
    crate::services::agent_local::session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}
