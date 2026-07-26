use super::subagent_completion::{
    persist_terminal_completion_with_after_report, persist_terminal_completion_with_hooks,
    SUBAGENT_COMPLETION_ERROR,
};
use super::subagent_orchestration::ParentSubagentOrchestrator;
use super::subagent_terminal_wait_test_support::{cleanup_parent, lock};
use super::{session_store, subagent_registry, subagent_status};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[tokio::test(start_paused = true)]
async fn missing_child_generic_report_stays_hidden_until_single_failure_signal() {
    let _guard = lock().await;
    let (parent, child) = parent_and_child("Missing child boundary").await;
    let orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    session_store::delete_one(&child.id)
        .await
        .expect("delete child before completion");
    let (saved_tx, saved_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let parent_id = parent.id.clone();
    let child_id = child.id.clone();
    let completion = tokio::spawn(async move {
        persist_terminal_completion_with_after_report(
            &parent_id,
            &child_id,
            "explorer",
            subagent_status::COMPLETED,
            "ignored",
            move |_outcome| async move {
                let _ = saved_tx.send(());
                let _ = release_rx.await;
            },
        )
        .await
    });
    saved_rx.await.expect("generic report saved");

    let mut waiter = spawn_waiter(orchestrator);
    assert!(tokio::time::timeout(Duration::from_millis(50), &mut waiter)
        .await
        .is_err());
    let _ = release_tx.send(());
    let completion_result = completion.await;
    let (wait_result, mut messages, mut orchestrator) = waiter.await.expect("join waiter");
    let prepare_result = orchestrator.prepare_for_model_request(&mut messages).await;
    let state = failure_state(&parent.id).await;
    cleanup_parent(&parent.id).await;

    let error = completion_result
        .expect("join completion")
        .err()
        .expect("missing child must fail");
    assert_eq!(error, SUBAGENT_COMPLETION_ERROR);
    assert_waiter_observed_failure(wait_result);
    assert_eq!(
        prepare_result,
        Err(SUBAGENT_COMPLETION_ERROR.into())
    );
    assert_single_failure_signal(state);
}

#[tokio::test(start_paused = true)]
async fn failed_child_save_generic_report_stays_hidden_until_single_failure_signal() {
    let _guard = lock().await;
    let (parent, child) = parent_and_child("Failed save boundary").await;
    let orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let child_path = crate::services::paths::data_dir()
        .join("agent-sessions")
        .join(format!("{}.json", child.id));
    let sabotage_path = child_path.clone();
    let (saved_tx, saved_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let parent_id = parent.id.clone();
    let child_id = child.id.clone();
    let completion = tokio::spawn(async move {
        persist_terminal_completion_with_hooks(
            &parent_id,
            &child_id,
            "explorer",
            subagent_status::COMPLETED,
            "ignored",
            move || async move {
                tokio::fs::remove_file(&sabotage_path)
                    .await
                    .expect("remove child file");
                tokio::fs::create_dir(&sabotage_path)
                    .await
                    .expect("block child save path");
            },
            move |_outcome| async move {
                let _ = saved_tx.send(());
                let _ = release_rx.await;
            },
        )
        .await
    });
    saved_rx.await.expect("generic report saved");

    let mut waiter = spawn_waiter(orchestrator);
    assert!(tokio::time::timeout(Duration::from_millis(50), &mut waiter)
        .await
        .is_err());
    let _ = release_tx.send(());
    let completion_result = completion.await;
    let (wait_result, mut messages, mut orchestrator) = waiter.await.expect("join waiter");
    let prepare_result = orchestrator.prepare_for_model_request(&mut messages).await;
    let state = failure_state(&parent.id).await;
    tokio::fs::remove_dir(&child_path)
        .await
        .expect("remove save blocker");
    session_store::save(&child).await.expect("restore child file");
    session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    cleanup_parent(&parent.id).await;

    let error = completion_result
        .expect("join completion")
        .err()
        .expect("failed save must fail");
    assert_eq!(error, SUBAGENT_COMPLETION_ERROR);
    assert_waiter_observed_failure(wait_result);
    assert_eq!(
        prepare_result,
        Err(SUBAGENT_COMPLETION_ERROR.into())
    );
    assert_single_failure_signal(state);
}

async fn parent_and_child(name: &str) -> (
    super::types_session::AgentSession,
    super::types_session::AgentSession,
) {
    let parent = session_store::create_full(name, "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    session_store::save(&child).await.expect("save child");
    subagent_registry::register(&parent.id, &child.id, CancellationToken::new())
        .await
        .expect("register child");
    (parent, child)
}

fn spawn_waiter(
    mut orchestrator: ParentSubagentOrchestrator,
) -> tokio::task::JoinHandle<(
    Result<bool, String>,
    Vec<super::types_ollama::ChatMessage>,
    ParentSubagentOrchestrator,
)> {
    tokio::spawn(async move {
        let mut messages = Vec::new();
        let result = orchestrator
            .after_no_tool_turn(&mut messages, CancellationToken::new())
            .await;
        (result, messages, orchestrator)
    })
}

async fn failure_state(
    parent_id: &str,
) -> super::subagent_terminal_signal::SubagentTerminalState {
    subagent_registry::terminal_state_for_parent(parent_id)
        .await
        .expect("terminal failure state")
}

fn assert_single_failure_signal(state: super::subagent_terminal_signal::SubagentTerminalState) {
    assert_eq!(state.sequence, 1);
    assert!(state.report_persistence_failed);
}

fn assert_waiter_observed_failure(result: Result<bool, String>) {
    assert!(
        result == Ok(true) || result == Err(SUBAGENT_COMPLETION_ERROR.to_string()),
        "le waiter doit se réveiller ou propager directement l'échec terminal"
    );
}
