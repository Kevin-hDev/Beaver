use super::subagent_completion::persist_terminal_completion;
use super::{session_store, subagent_hidden_reports, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn full_parent_report_queue_persists_a_retryable_overflow_report() {
    let parent = session_store::create_full("Parent full queue", "llama3", "ollama", false, None)
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
    for index in 0..subagent_hidden_reports::MAX_PENDING_REPORTS {
        let report = subagent_hidden_reports::build_report(
            format!("previous-{index}"),
            "Agent".into(),
            "explorer".into(),
            subagent_status::COMPLETED.into(),
            "Rapport précédent".into(),
        );
        subagent_hidden_reports::append(&parent.id, report)
            .await
            .expect("fill report queue");
    }

    let result = persist_terminal_completion(
        &parent.id,
        &child.id,
        "explorer",
        subagent_status::COMPLETED,
        "Nouveau rapport",
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(
        subagent_hidden_reports::peek_reports(&parent.id)
            .await
            .len(),
        subagent_hidden_reports::MAX_PENDING_REPORTS
    );
    assert!(
        !subagent_registry::terminal_state_for_parent(&parent.id)
            .await
            .expect("terminal state")
            .report_persistence_failed
    );
    let overflow = super::subagent_report_overflow::pending_for_parent(&parent.id).await;
    assert_eq!(overflow.len(), 1);
    assert_eq!(overflow[0].child_session_id, child.id);
    assert_eq!(
        session_store::get(&child.id)
            .await
            .expect("saved child")
            .subagent_status
            .as_deref(),
        Some(subagent_status::COMPLETED)
    );
    session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}
