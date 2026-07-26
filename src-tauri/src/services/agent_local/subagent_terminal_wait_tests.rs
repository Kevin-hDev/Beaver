use super::subagent_orchestration::ParentSubagentOrchestrator;
use super::subagent_terminal_wait_test_support::{cleanup_parent, lock};
use super::{session_store, subagent_hidden_reports, subagent_registry};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

async fn parent_session(name: &str) -> super::types_session::AgentSession {
    session_store::create_full(name, "llama3", "ollama", false, None)
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

#[tokio::test(start_paused = true)]
async fn terminal_report_wakes_parent_immediately() {
    let _guard = lock().await;
    let parent = parent_session("Terminal report wait").await;
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let child_id = uuid::Uuid::new_v4().to_string();
    subagent_registry::register(&parent.id, &child_id, CancellationToken::new())
        .await
        .expect("register child");
    let waiter = tokio::spawn(async move {
        let mut messages = Vec::new();
        let outcome = orchestrator
            .after_no_tool_turn(&mut messages, CancellationToken::new())
            .await;
        (outcome, messages)
    });
    tokio::task::yield_now().await;

    subagent_hidden_reports::append(&parent.id, report(&child_id))
        .await
        .expect("append report");
    subagent_registry::complete_child(
        &child_id,
        subagent_registry::SubagentTerminalKind::ReportPersisted,
    )
    .await
    .expect("complete child");

    let (outcome, messages) = tokio::time::timeout(Duration::from_millis(1), waiter)
        .await
        .expect("terminal event must wake immediately")
        .expect("join waiter");
    assert_eq!(outcome, Ok(true));
    assert!(messages.iter().any(|message| {
        message
            .content
            .starts_with(super::subagent_report_context::SUBAGENT_REPORT_CONTEXT_PREFIX)
    }));
    cleanup_parent(&parent.id).await;
}

#[tokio::test(start_paused = true)]
async fn terminal_failure_wakes_parent_immediately() {
    let _guard = lock().await;
    let parent = parent_session("Terminal failure wait").await;
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let child_id = uuid::Uuid::new_v4().to_string();
    subagent_registry::register(&parent.id, &child_id, CancellationToken::new())
        .await
        .expect("register child");
    let waiter = tokio::spawn(async move {
        orchestrator
            .after_no_tool_turn(&mut Vec::new(), CancellationToken::new())
            .await
    });
    tokio::task::yield_now().await;

    subagent_registry::complete_child(
        &child_id,
        subagent_registry::SubagentTerminalKind::ReportPersistenceFailed,
    )
    .await
    .expect("complete child failure");

    let outcome = tokio::time::timeout(Duration::from_millis(1), waiter)
        .await
        .expect("terminal failure must wake immediately")
        .expect("join waiter");
    assert_eq!(
        outcome,
        Err(super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string())
    );
    cleanup_parent(&parent.id).await;
}

#[tokio::test(start_paused = true)]
async fn first_report_resumes_once_then_waits_for_the_second_child() {
    let _guard = lock().await;
    let parent = parent_session("Two terminal reports").await;
    let mut orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let first_child = uuid::Uuid::new_v4().to_string();
    let second_child = uuid::Uuid::new_v4().to_string();
    for child_id in [&first_child, &second_child] {
        subagent_registry::register(&parent.id, child_id, CancellationToken::new())
            .await
            .expect("register child");
    }
    let first_waiter = tokio::spawn(async move {
        let mut messages = Vec::new();
        let outcome = orchestrator
            .after_no_tool_turn(&mut messages, CancellationToken::new())
            .await;
        (outcome, messages, orchestrator)
    });
    tokio::task::yield_now().await;

    subagent_hidden_reports::append(&parent.id, report(&first_child))
        .await
        .expect("append first report");
    subagent_registry::complete_child(
        &first_child,
        subagent_registry::SubagentTerminalKind::ReportPersisted,
    )
    .await
    .expect("complete first child");
    let (first_outcome, mut messages, mut orchestrator) =
        tokio::time::timeout(Duration::from_millis(1), first_waiter)
            .await
            .expect("first report must wake immediately")
            .expect("join first waiter");
    assert_eq!(first_outcome, Ok(true));
    orchestrator
        .complete_model_request(true, &CancellationToken::new(), &messages)
        .await
        .expect("acknowledge first report");

    let second_waiter = tokio::spawn(async move {
        let outcome = orchestrator
            .after_no_tool_turn(&mut messages, CancellationToken::new())
            .await;
        (outcome, messages)
    });
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(60 * 60)).await;
    tokio::task::yield_now().await;
    assert!(
        !second_waiter.is_finished(),
        "the parent resumed twice for the first report"
    );

    subagent_hidden_reports::append(&parent.id, report(&second_child))
        .await
        .expect("append second report");
    subagent_registry::complete_child(
        &second_child,
        subagent_registry::SubagentTerminalKind::ReportPersisted,
    )
    .await
    .expect("complete second child");
    let (second_outcome, _) = tokio::time::timeout(Duration::from_millis(1), second_waiter)
        .await
        .expect("second report must wake immediately")
        .expect("join second waiter");
    assert_eq!(second_outcome, Ok(true));
    cleanup_parent(&parent.id).await;
}
