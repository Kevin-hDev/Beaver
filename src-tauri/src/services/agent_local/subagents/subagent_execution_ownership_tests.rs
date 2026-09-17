use super::{session_store, subagent_hidden_reports, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn stale_panic_cannot_terminalize_redeployed_child_in_same_parent_run() {
    let (parent, mut child, sibling) = sessions("panic").await;
    let old = register(&parent.id, &child.id).await;
    let _sibling = register(&parent.id, &sibling.id).await;
    save_run(&mut child, &old.run_id).await;
    subagent_registry::unregister(&child.id).await;
    let new = register(&parent.id, &child.id).await;

    let recovered = super::subagent_panic_supervisor::recover_panicked_completion(
        &parent.id,
        &child.id,
        "explorer",
        &old.run_id,
        &old.execution_id,
        None,
        None,
    )
    .await;

    assert!(!recovered);
    assert_new_execution_untouched(&parent.id, &child.id, &new).await;
    cleanup(&parent.id, &child.id, &sibling.id).await;
}

#[tokio::test]
async fn stale_preparation_failure_is_a_noop_for_redeployed_child() {
    let (parent, mut child, sibling) = sessions("preparation").await;
    let old = register(&parent.id, &child.id).await;
    let _sibling = register(&parent.id, &sibling.id).await;
    save_run(&mut child, &old.run_id).await;
    subagent_registry::unregister(&child.id).await;
    let new = register(&parent.id, &child.id).await;

    let reported = super::subagent_task::finish_preparation_failure(
        &parent.id,
        &child.id,
        "explorer",
        &old.run_id,
        &old.execution_id,
        None,
    )
    .await;

    assert!(!reported);
    assert_new_execution_untouched(&parent.id, &child.id, &new).await;
    cleanup(&parent.id, &child.id, &sibling.id).await;
}

async fn assert_new_execution_untouched(
    parent_id: &str,
    child_id: &str,
    expected: &subagent_registry::RegisteredSubagent,
) {
    let saved = session_store::get(child_id).await.expect("saved child");
    assert_eq!(saved.subagent_status.as_deref(), Some(subagent_status::RUNNING));
    assert!(subagent_registry::owns_execution(child_id, &expected.run_id, &expected.execution_id).await);
    assert!(subagent_hidden_reports::peek_reports(parent_id).await.is_empty());
}

async fn sessions(
    suffix: &str,
) -> (
    super::types_session::AgentSession,
    super::types_session::AgentSession,
    super::types_session::AgentSession,
) {
    let parent = session_store::create_full(
        &format!("Parent {suffix}"),
        "llama3",
        "ollama",
        false,
        None,
    )
    .await
    .expect("create parent");
    let child = create_child(&parent.id, &format!("Child {suffix}")).await;
    let sibling = create_child(&parent.id, &format!("Sibling {suffix}")).await;
    (parent, child, sibling)
}

async fn create_child(parent_id: &str, name: &str) -> super::types_session::AgentSession {
    let mut child = session_store::create_full(name, "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent_id.into());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    session_store::save(&child).await.expect("save child");
    child
}

async fn register(parent_id: &str, child_id: &str) -> subagent_registry::RegisteredSubagent {
    subagent_registry::register_execution(parent_id, child_id, CancellationToken::new())
        .await
        .expect("register execution")
}

async fn save_run(child: &mut super::types_session::AgentSession, run_id: &str) {
    child.subagent_run_id = Some(run_id.to_string());
    session_store::save(child).await.expect("save run");
}

async fn cleanup(parent_id: &str, child_id: &str, sibling_id: &str) {
    subagent_registry::unregister(child_id).await;
    subagent_registry::unregister(sibling_id).await;
    for id in [child_id, sibling_id, parent_id] {
        session_store::delete_one(id).await.expect("delete session");
    }
}
