use super::*;

async fn parent_session() -> super::super::types_session::AgentSession {
    super::super::session_store::create_full(
        "Parent orchestration",
        "llama3",
        "ollama",
        false,
        None,
    )
    .await
    .expect("create parent session")
}

fn report(child_id: &str) -> super::super::types_session::SubagentHiddenReport {
    super::super::subagent_hidden_reports::build_report(
        child_id.to_string(),
        "Geminitor".into(),
        "explorer".into(),
        "completed".into(),
        format!("Rapport {child_id}"),
    )
}

#[tokio::test]
async fn report_batch_is_injected_only_once_while_acknowledgement_is_pending() {
    let parent = parent_session().await;
    super::super::subagent_hidden_reports::append(&parent.id, report("child-a"))
        .await
        .expect("append first report");
    super::super::subagent_hidden_reports::append(&parent.id, report("child-b"))
        .await
        .expect("append second report");
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let mut messages = Vec::new();

    assert!(orchestrator.inject_pending_reports(&mut messages).await);
    assert!(!orchestrator.inject_pending_reports(&mut messages).await);

    let batches = messages
        .iter()
        .filter(|message| {
            message
                .content
                .starts_with(super::super::subagent_hidden_reports::SUBAGENT_REPORT_CONTEXT_PREFIX)
        })
        .count();
    assert_eq!(batches, 1);
    assert_eq!(
        super::super::subagent_hidden_reports::peek_reports(&parent.id)
            .await
            .len(),
        2
    );
    super::super::session_store::delete_one(&parent.id)
        .await
        .expect("delete parent session");
}

#[tokio::test]
async fn failed_cancelled_or_replaced_model_request_keeps_reports() {
    let parent = parent_session().await;
    super::super::subagent_hidden_reports::append(&parent.id, report("child"))
        .await
        .expect("append report");
    let mut failed_request = ParentSubagentOrchestrator::new(&parent.id).await;
    let mut messages = Vec::new();
    assert!(failed_request.inject_pending_reports(&mut messages).await);
    drop(failed_request);

    let mut replacement = ParentSubagentOrchestrator::new(&parent.id).await;
    let mut replacement_messages = Vec::new();
    assert!(
        replacement
            .inject_pending_reports(&mut replacement_messages)
            .await
    );
    replacement
        .complete_model_request(false, &CancellationToken::new(), &replacement_messages)
        .await
        .expect("record cancelled outcome");

    assert_eq!(
        super::super::subagent_hidden_reports::peek_reports(&parent.id)
            .await
            .len(),
        1
    );
    super::super::session_store::delete_one(&parent.id)
        .await
        .expect("delete parent session");
}

#[tokio::test]
async fn successful_model_outcome_acknowledges_injected_reports() {
    let parent = parent_session().await;
    super::super::subagent_hidden_reports::append(&parent.id, report("child"))
        .await
        .expect("append report");
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let mut messages = Vec::new();
    assert!(orchestrator.inject_pending_reports(&mut messages).await);

    orchestrator
        .complete_model_request(true, &CancellationToken::new(), &messages)
        .await
        .expect("acknowledge successful outcome");

    assert!(
        super::super::subagent_hidden_reports::peek_reports(&parent.id)
            .await
            .is_empty()
    );
    super::super::session_store::delete_one(&parent.id)
        .await
        .expect("delete parent session");
}

#[tokio::test]
async fn report_persistence_failure_signal_stops_parent_without_finalizing() {
    let parent = parent_session().await;
    let child_id = uuid::Uuid::new_v4().to_string();
    super::super::subagent_registry::register(&parent.id, &child_id, CancellationToken::new())
        .await
        .expect("register child");
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let mut messages = Vec::new();
    orchestrator
        .prepare_for_model_request(&mut messages)
        .await
        .expect("prepare parent request");
    super::super::subagent_registry::complete_child(
        &child_id,
        super::super::subagent_registry::SubagentTerminalKind::ReportPersistenceFailed,
    )
    .await
    .expect("atomic terminal failure");

    let error = orchestrator
        .after_no_tool_turn(&mut messages, CancellationToken::new())
        .await
        .expect_err("persistence failure must stop the parent");

    assert_eq!(
        error,
        super::super::subagent_completion::SUBAGENT_COMPLETION_ERROR
    );
    super::super::session_store::delete_one(&parent.id)
        .await
        .expect("delete parent session");
}

#[tokio::test]
async fn active_registry_entry_survives_missing_child_session() {
    let parent = parent_session().await;
    let child_id = uuid::Uuid::new_v4().to_string();
    super::super::subagent_registry::register(&parent.id, &child_id, CancellationToken::new())
        .await
        .expect("register missing child");

    let active = current_turn_active_ids(&parent.id).await;

    super::super::subagent_registry::unregister(&child_id).await;
    super::super::session_store::delete_one(&parent.id)
        .await
        .expect("delete parent session");
    assert_eq!(active.len(), 1);
}

#[test]
fn api_and_ollama_ack_only_after_a_successful_model_outcome() {
    assert_shared_model_outcomes(include_str!("../llm/agent_loop_request.rs"));
    assert_shared_model_outcomes(include_str!("agent_loop_ollama_request.rs"));
}

#[test]
fn chat_entrypoint_does_not_inject_reports_before_the_model_loop() {
    let source = include_str!("../../commands/agent_chat.rs");
    assert!(!source.contains("inject_hidden_subagent_reports"));
}

fn assert_shared_model_outcomes(source: &str) {
    assert!(source.contains("complete_model_request(!interrupted, &completion_cancel, params.messages)"));
    let model_result = source
        .find("record_model_result")
        .expect("model result is recorded");
    let acknowledgement = source
        .find("complete_model_request(!interrupted, &completion_cancel, params.messages)")
        .expect("successful model outcome is finalized");
    assert!(model_result < acknowledgement);
}
