use super::{
    session_store, subagent_hidden_reports, subagent_panic_supervisor, subagent_registry,
    subagent_status,
};
use tokio_util::sync::CancellationToken;

async fn session(name: &str) -> super::types_session::AgentSession {
    session_store::create_full(name, "llama3", "ollama", false, None)
        .await
        .expect("create session")
}

#[tokio::test]
async fn panic_persists_generic_failure_and_leaves_no_registry_ghost() {
    let parent = session("Parent panic").await;
    let mut child = session("Geminitor").await;
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    session_store::save(&child).await.expect("save child");
    let registered = subagent_registry::register_execution(
        &parent.id,
        &child.id,
        CancellationToken::new(),
    )
        .await
        .expect("register child");
    child.subagent_run_id = Some(registered.run_id.clone());
    session_store::save(&child).await.expect("save run id");
    let mut signal = subagent_registry::subscribe_for_parent(&parent.id)
        .await
        .expect("subscribe signal");
    let parent_id = parent.id.clone();
    let child_id = child.id.clone();
    let run_id = registered.run_id;
    let execution_id = registered.execution_id;

    subagent_panic_supervisor::run_guarded(
        async { panic!("internal panic must stay private") },
        move || async move {
            subagent_panic_supervisor::recover_panicked_completion(
                &parent_id,
                &child_id,
                "explorer",
                &run_id,
                &execution_id,
                None,
                None,
            )
            .await;
        },
    )
    .await;

    signal.changed().await.expect("panic completion signal");
    assert!(subagent_registry::active_children_for_parent(&parent.id)
        .await
        .is_empty());
    let saved_child = session_store::get(&child.id).await.expect("saved child");
    assert_eq!(
        saved_child.subagent_status.as_deref(),
        Some(subagent_status::FAILED)
    );
    let reports = subagent_hidden_reports::peek_reports(&parent.id).await;
    assert_eq!(reports.len(), 1);
    assert_eq!(
        reports[0].summary,
        subagent_panic_supervisor::SUBAGENT_PANIC_SUMMARY
    );
    assert!(!reports[0].summary.contains("internal panic"));
    session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}
