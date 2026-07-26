use super::subagent_orchestration::ParentSubagentOrchestrator;
use super::{session_store, subagent_hidden_reports, subagent_registry};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn terminal_failure_blocks_before_provider_preparation() {
    let parent = session_store::create_full("Parent pre-provider failure", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let child_id = uuid::Uuid::new_v4().to_string();
    subagent_registry::register(&parent.id, &child_id, CancellationToken::new())
        .await
        .expect("register child");
    subagent_registry::complete_child(
        &child_id,
        subagent_registry::SubagentTerminalKind::ReportPersistenceFailed,
    )
    .await
    .expect("signal failed persistence");
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;

    let error = orchestrator
        .prepare_for_model_request(&mut Vec::new())
        .await
        .expect_err("provider preparation must stop");

    assert_eq!(error, super::subagent_completion::SUBAGENT_COMPLETION_ERROR);
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn terminal_failure_blocks_completion_without_pending_report_ids() {
    let parent = session_store::create_full("Parent empty failed delivery", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let child_id = uuid::Uuid::new_v4().to_string();
    subagent_registry::register(&parent.id, &child_id, CancellationToken::new())
        .await
        .expect("register child");
    subagent_registry::complete_child(
        &child_id,
        subagent_registry::SubagentTerminalKind::ReportPersistenceFailed,
    )
    .await
    .expect("signal failed persistence");
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;

    let error = orchestrator
        .complete_model_request(true, &CancellationToken::new(), &[])
        .await
        .expect_err("empty report set must not hide terminal failure");

    assert_eq!(error, super::subagent_completion::SUBAGENT_COMPLETION_ERROR);
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn persistence_failure_is_never_acknowledged_as_a_successful_report() {
    let parent = session_store::create_full("Parent failed delivery", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let child_id = uuid::Uuid::new_v4().to_string();
    subagent_registry::register(&parent.id, &child_id, CancellationToken::new())
        .await
        .expect("register child");
    subagent_hidden_reports::append(
        &parent.id,
        subagent_hidden_reports::build_report(
            child_id.clone(),
            "Geminitor".into(),
            "explorer".into(),
            "failed".into(),
            "Échec générique".into(),
        ),
    )
    .await
    .expect("append generic report");
    subagent_registry::complete_child(
        &child_id,
        subagent_registry::SubagentTerminalKind::ReportPersistenceFailed,
    )
    .await
    .expect("signal failed persistence");
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let mut messages = Vec::new();
    assert!(orchestrator.inject_pending_reports(&mut messages).await);

    assert!(orchestrator
        .complete_model_request(true, &CancellationToken::new(), &messages)
        .await
        .is_err());
    assert_eq!(subagent_hidden_reports::peek_reports(&parent.id).await.len(), 1);
    assert!(
        subagent_registry::terminal_state_for_parent(&parent.id)
            .await
            .expect("durable terminal failure")
            .report_persistence_failed
    );
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}
