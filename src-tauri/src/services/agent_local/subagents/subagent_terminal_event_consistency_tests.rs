use super::{session_store, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn initial_save_failure_reports_one_coherent_failure_outcome() {
    let parent = session_store::create_full("Parent event failure", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::COMPLETED.into());
    session_store::save(&child).await.expect("save child");
    subagent_registry::register(&parent.id, &child.id, CancellationToken::new())
        .await
        .expect("register child");
    let child_id = child.id.clone();
    let (event_tx, event_rx) = tokio::sync::oneshot::channel();

    super::subagent_completion_boundary::complete_failure(
        &parent.id,
        &child_id,
        "explorer",
        &mut child,
        true,
        false,
        move |outcome| async move {
            let _ = event_tx.send((outcome.success, outcome.status, outcome.summary));
        },
    )
    .await
    .expect("persist generic failure");

    let emitted = event_rx.await.expect("terminal event");
    let saved_child = session_store::get(&child.id).await.expect("saved child");
    let reports = super::subagent_hidden_reports::peek_reports(&parent.id).await;
    assert_eq!(
        emitted,
        (
            false,
            subagent_status::FAILED.to_string(),
            super::subagent_completion::SUBAGENT_COMPLETION_ERROR.to_string(),
        )
    );
    assert_eq!(saved_child.subagent_status.as_deref(), Some(subagent_status::FAILED));
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].status, subagent_status::FAILED);
    assert_eq!(reports[0].summary, super::subagent_completion::SUBAGENT_COMPLETION_ERROR);
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}
