use super::subagent_completion::{
    persist_terminal_completion_with_after_report, SUBAGENT_COMPLETION_ERROR,
};
use super::subagent_orchestration::ParentSubagentOrchestrator;
use super::subagent_terminal_wait_test_support::{cleanup_parent, lock};
use super::{session_store, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test(start_paused = true)]
async fn durable_report_cannot_be_acknowledged_before_terminal_signal() {
    let _guard = lock().await;
    let parent = session_store::create_full("Parent signal race", "llama3", "ollama", false, None)
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
    let orchestrator = ParentSubagentOrchestrator::new(&parent.id).await;
    let (saved_tx, saved_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let completion_parent = parent.id.clone();
    let completion_child = child.id.clone();
    let completion = tokio::spawn(async move {
        persist_terminal_completion_with_after_report(
            &completion_parent,
            &completion_child,
            "explorer",
            subagent_status::COMPLETED,
            "Rapport final",
            move |_outcome| async move {
                let _ = saved_tx.send(());
                let _ = release_rx.await;
            },
        )
        .await
    });
    saved_rx.await.expect("report saved hook");

    let mut waiter = tokio::spawn(async move {
        let mut orchestrator = orchestrator;
        let mut messages = Vec::new();
        let outcome = orchestrator
            .after_no_tool_turn(&mut messages, CancellationToken::new())
            .await;
        (outcome, messages, orchestrator)
    });
    let early_wait = tokio::time::timeout(std::time::Duration::from_millis(50), &mut waiter).await;
    let woke_before_signal = early_wait.is_ok();

    let (outcome, messages, mut orchestrator) = if let Ok(joined) = early_wait {
        let (outcome, messages, mut orchestrator) = joined.expect("join early waiter");
        orchestrator
            .complete_model_request(true, &CancellationToken::new(), &messages)
            .await
            .expect("early acknowledgement");
        let _ = release_tx.send(());
        completion
            .await
            .expect("join completion")
            .expect("finish completion");
        (outcome, messages, orchestrator)
    } else {
        let _ = release_tx.send(());
        completion
            .await
            .expect("join completion")
            .expect("finish completion");
        waiter.await.expect("join signalled waiter")
    };
    assert_eq!(outcome, Ok(true));
    if !woke_before_signal {
        orchestrator
            .complete_model_request(true, &CancellationToken::new(), &messages)
            .await
            .expect("acknowledge report");
    }
    let final_boundary = orchestrator
        .after_no_tool_turn(&mut Vec::new(), CancellationToken::new())
        .await;

    assert!(!woke_before_signal, "report escaped before terminal signal");
    assert_ne!(
        final_boundary,
        Err(SUBAGENT_COMPLETION_ERROR.to_string()),
        "successful acknowledgement produced a false terminal failure"
    );
    session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    cleanup_parent(&parent.id).await;
}
