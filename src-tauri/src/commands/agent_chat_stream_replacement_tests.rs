use super::agent_chat_streams::replace_active_stream;
use crate::services::agent_local::subagent_registry;
use crate::ActiveStreams;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};
use tokio_util::sync::CancellationToken;

#[test]
fn chat_stream_uses_the_tested_replacement_path() {
    let source = include_str!("agent_chat.rs");

    assert!(
        source.contains("agent_chat_streams::replace_active_stream"),
        "chat_stream contourne encore la frontière de remplacement testable"
    );
}

#[test]
fn active_user_message_uses_the_current_stream_inbox() {
    let backend = include_str!("agent_chat_queue.rs");

    assert!(backend.contains("pub async fn queue_agent_message"));
    assert!(backend.contains("inbox.enqueue"));
}

#[tokio::test]
async fn later_start_wins_while_previous_cancellation_is_suspended() {
    let streams = Arc::new(ActiveStreams(Mutex::new(HashMap::new())));
    let session_id = id();
    let child_id = id();
    let old_owner = CancellationToken::new();
    let child_cancel = CancellationToken::new();
    streams.0.lock().await.insert(
        session_id.clone(),
        (old_owner.clone(), 0, "request-old".to_string(), inbox()),
    );
    subagent_registry::register_execution_for_parent_stream(
        &session_id,
        &child_id,
        child_cancel.clone(),
        None,
        &old_owner,
    )
    .await
    .expect("register child");

    let first_cancel = CancellationToken::new();
    let second_cancel = CancellationToken::new();
    let (at_boundary_tx, at_boundary_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let first_task = {
        let streams = streams.clone();
        let session_id = session_id.clone();
        let first_cancel = first_cancel.clone();
        tokio::spawn(async move {
            replace_active_stream(
                &streams,
                &session_id,
                first_cancel,
                1,
                inbox(),
                move |(old_cancel, _, _, _)| async move {
                    let _ = at_boundary_tx.send(());
                    let _ = release_rx.await;
                    old_cancel.cancel();
                },
                || async { "request-a".to_string() },
            )
            .await
        })
    };
    at_boundary_rx
        .await
        .expect("first start reached cancellation");

    replace_active_stream(
        &streams,
        &session_id,
        second_cancel.clone(),
        2,
        inbox(),
        |(old_cancel, _, _, _)| async move { old_cancel.cancel() },
        || async { "request-b".to_string() },
    )
    .await
    .expect("second start");
    release_tx.send(()).expect("release first start");
    let first_result = first_task.await.expect("join first start");

    let (tracked_count, tracked_generation, tracked_request) = {
        let map = streams.0.lock().await;
        let (_, generation, request_id, _) = map.get(&session_id).expect("tracked winner");
        (map.len(), *generation, request_id.clone())
    };
    second_cancel.cancel();
    subagent_registry::cancel_stopped_parent_stream_children(&session_id).await;
    let winner_owns_child = child_cancel.is_cancelled();
    subagent_registry::unregister(&child_id).await;

    assert!(
        first_result.is_err(),
        "le démarrage perdant continue vers le writer"
    );
    assert_eq!(tracked_count, 1);
    assert_eq!(tracked_generation, 2);
    assert_eq!(tracked_request, "request-b");
    assert!(first_cancel.is_cancelled(), "le writer perdant reste actif");
    assert!(
        winner_owns_child,
        "l'enfant appartient encore au writer perdant"
    );
}

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn inbox() -> Arc<crate::services::agent_local::parent_message_inbox::ParentMessageInbox> {
    Arc::new(crate::services::agent_local::parent_message_inbox::ParentMessageInbox::new())
}
