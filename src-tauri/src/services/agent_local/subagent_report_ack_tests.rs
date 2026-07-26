use super::{session_locks, session_store, subagent_hidden_reports};
use tokio_util::sync::CancellationToken;

async fn parent_report() -> (
    super::types_session::AgentSession,
    super::types_session::SubagentHiddenReport,
) {
    let parent = session_store::create_full("Parent ack order", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let report = subagent_hidden_reports::build_report(
        "child".into(),
        "Geminitor".into(),
        "explorer".into(),
        "completed".into(),
        "Rapport durable".into(),
    );
    subagent_hidden_reports::append(&parent.id, report.clone())
        .await
        .expect("append report");
    (parent, report)
}

#[tokio::test]
async fn cancellation_queued_before_ack_keeps_report() {
    let (parent, report) = parent_report().await;
    let lock = session_store::lock_session(&parent.id).await;
    let guard = lock.lock().await;
    let cancel = CancellationToken::new();
    let cancel_task_token = cancel.clone();
    let cancel_parent = parent.id.clone();
    let (cancel_started_tx, cancel_started_rx) = tokio::sync::oneshot::channel();
    let cancellation = tokio::spawn(async move {
        let _ = cancel_started_tx.send(());
        session_locks::cancel_with_lock(&cancel_parent, &cancel_task_token).await;
    });
    cancel_started_rx.await.expect("cancellation starts");
    tokio::task::yield_now().await;
    let ack_parent = parent.id.clone();
    let ack_report_id = report.id.clone();
    let ack_cancel = cancel.clone();
    let acknowledgement = tokio::spawn(async move {
        subagent_hidden_reports::acknowledge_reports_cancellable(
            &ack_parent,
            &[ack_report_id],
            &ack_cancel,
        )
        .await
    });

    drop(guard);
    cancellation.await.expect("cancellation task");
    assert!(acknowledgement.await.expect("ack task").is_err());
    assert_eq!(
        subagent_hidden_reports::peek_reports(&parent.id).await,
        vec![report]
    );
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn ack_queued_before_cancellation_commits_delivery() {
    let (parent, report) = parent_report().await;
    let lock = session_store::lock_session(&parent.id).await;
    let guard = lock.lock().await;
    let cancel = CancellationToken::new();
    let ack_parent = parent.id.clone();
    let ack_report_id = report.id.clone();
    let ack_cancel = cancel.clone();
    let (ack_started_tx, ack_started_rx) = tokio::sync::oneshot::channel();
    let acknowledgement = tokio::spawn(async move {
        let _ = ack_started_tx.send(());
        subagent_hidden_reports::acknowledge_reports_cancellable(
            &ack_parent,
            &[ack_report_id],
            &ack_cancel,
        )
        .await
    });
    ack_started_rx.await.expect("ack starts");
    tokio::task::yield_now().await;
    let cancel_parent = parent.id.clone();
    let cancel_task_token = cancel.clone();
    let cancellation = tokio::spawn(async move {
        session_locks::cancel_with_lock(&cancel_parent, &cancel_task_token).await;
    });

    drop(guard);
    acknowledgement
        .await
        .expect("ack task")
        .expect("ack wins lock order");
    cancellation.await.expect("cancellation task");
    assert!(subagent_hidden_reports::peek_reports(&parent.id)
        .await
        .is_empty());
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[test]
fn parent_stream_cancellation_paths_use_the_delivery_lock() {
    let source = include_str!("../../commands/agent_chat.rs");
    let stop_source = include_str!("../../commands/agent_chat_cancel.rs");

    assert_eq!(
        source.matches("cancel_with_lock").count()
            + stop_source.matches("cancel_with_lock").count(),
        2
    );
    assert!(!source.contains("old_token.cancel()"));
    assert!(!source.contains("token.cancel();"));
    assert!(!stop_source.contains("token.cancel();"));
}
