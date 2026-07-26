use super::subagent_orchestration::ParentSubagentOrchestrator;
use super::{session_store, subagent_hidden_reports, subagent_registry};
use tokio_util::sync::CancellationToken;

async fn parent_session() -> super::types_session::AgentSession {
    session_store::create_full("Parent race", "llama3", "ollama", false, None)
        .await
        .expect("create parent")
}

fn report(child_id: &str) -> super::types_session::SubagentHiddenReport {
    subagent_hidden_reports::build_report(
        child_id.to_string(),
        "Geminitor".into(),
        "explorer".into(),
        "completed".into(),
        format!("Rapport {child_id}"),
    )
}

#[tokio::test]
async fn sequential_waves_consume_old_terminal_state_before_registering_new_signal() {
    let parent = parent_session().await;
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let first_child = uuid::Uuid::new_v4().to_string();
    subagent_registry::register(&parent.id, &first_child, CancellationToken::new())
        .await
        .expect("register first wave");
    let first_generation = subagent_registry::terminal_state_for_parent(&parent.id)
        .await
        .expect("first signal")
        .generation;
    subagent_hidden_reports::append(&parent.id, report(&first_child))
        .await
        .expect("append first report");
    subagent_registry::complete_child(
        &first_child,
        subagent_registry::SubagentTerminalKind::ReportPersisted,
    )
    .await
    .expect("complete first wave");
    let mut messages = Vec::new();
    orchestrator
        .prepare_for_model_request(&mut messages)
        .await
        .expect("prepare first report");
    orchestrator
        .complete_model_request(true, &CancellationToken::new(), &messages)
        .await
        .expect("ack first wave");

    let second_child = uuid::Uuid::new_v4().to_string();
    subagent_registry::register(&parent.id, &second_child, CancellationToken::new())
        .await
        .expect("register second wave after consume");
    let second_generation = subagent_registry::terminal_state_for_parent(&parent.id)
        .await
        .expect("second signal")
        .generation;
    assert_ne!(first_generation, second_generation);

    subagent_registry::unregister(&second_child).await;
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn terminal_boundary_without_persisted_report_fails_closed() {
    let parent = parent_session().await;
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let child_id = uuid::Uuid::new_v4().to_string();
    subagent_registry::register(&parent.id, &child_id, CancellationToken::new())
        .await
        .expect("register child");
    subagent_registry::complete_child(
        &child_id,
        subagent_registry::SubagentTerminalKind::ReportPersisted,
    )
    .await
    .expect("complete child");

    let error = orchestrator
        .after_no_tool_turn(&mut Vec::new(), CancellationToken::new())
        .await
        .expect_err("missing terminal report must block finalization");

    assert_eq!(error, super::subagent_completion::SUBAGENT_COMPLETION_ERROR);
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn cancellation_after_stream_completion_keeps_unacknowledged_report() {
    let parent = parent_session().await;
    subagent_hidden_reports::append(&parent.id, report("child"))
        .await
        .expect("append report");
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let mut messages = Vec::new();
    assert!(orchestrator.inject_pending_reports(&mut messages).await);
    let cancel = CancellationToken::new();
    cancel.cancel();

    assert!(orchestrator
        .complete_model_request(true, &cancel, &messages)
        .await
        .is_err());
    assert_eq!(subagent_hidden_reports::peek_reports(&parent.id).await.len(), 1);
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn missing_report_from_real_payload_blocks_acknowledgement() {
    let parent = parent_session().await;
    subagent_hidden_reports::append(&parent.id, report("child"))
        .await
        .expect("append report");
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let mut messages = Vec::new();
    assert!(orchestrator.inject_pending_reports(&mut messages).await);
    messages.retain(|message| {
        !message.content.starts_with(
            super::subagent_report_context::SUBAGENT_REPORT_CONTEXT_PREFIX,
        )
    });

    assert!(orchestrator
        .complete_model_request(true, &CancellationToken::new(), &messages)
        .await
        .is_err());
    assert_eq!(subagent_hidden_reports::peek_reports(&parent.id).await.len(), 1);
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn report_policy_and_body_match_api_and_ollama_payloads() {
    let parent = parent_session().await;
    subagent_hidden_reports::append(&parent.id, report("child"))
        .await
        .expect("append report");
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let mut messages = vec![super::types_ollama::ChatMessage {
        role: "user".into(),
        content: "Question".into(),
        ..Default::default()
    }];
    orchestrator
        .prepare_for_model_request(&mut messages)
        .await
        .expect("prepare payload");
    super::context_budget::prepare_for_request(&mut messages, 12_000)
        .expect("payload budget");
    let ollama = super::agent_loop_support::build_request(
        "llama3",
        &messages,
        &[],
        super::types_ollama::OllamaThink::Bool(false),
    );
    let api = crate::services::llm::stream_convert::messages_to_openai(&messages, "openai");

    assert!(ollama.messages[0]
        .content
        .starts_with(super::subagent_report_context::SUBAGENT_REPORT_POLICY_PREFIX));
    assert_eq!(api[0]["role"], "system");
    assert!(api[0]["content"]
        .as_str()
        .is_some_and(|content| content.starts_with(
            super::subagent_report_context::SUBAGENT_REPORT_POLICY_PREFIX
        )));
    let expected = messages
        .iter()
        .find(|message| message.content.starts_with(
            super::subagent_report_context::SUBAGENT_REPORT_CONTEXT_PREFIX,
        ))
        .expect("report payload")
        .content
        .as_str();
    assert!(ollama.messages.iter().any(|message| message.content == expected));
    assert!(api.iter().any(|message| message["content"] == expected));
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}
