use super::{session_store, subagent_instruction_delivery, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn drain_rechecks_execution_after_waiting_for_child_lock() {
    let (parent, mut child, sibling) = sessions("drain race").await;
    let old = register(&parent.id, &child.id).await;
    let _sibling = register(&parent.id, &sibling.id).await;
    child.subagent_run_id = Some(old.run_id.clone());
    child.subagent_queued_prompts.push("ancienne correction".into());
    session_store::save(&child).await.expect("save old execution");
    let child_lock = session_store::lock_session(&child.id).await;
    let guard = child_lock.lock().await;
    let (read_tx, read_rx) = tokio::sync::oneshot::channel();
    let child_id = child.id.clone();
    let drain = tokio::spawn(async move {
        let mut context = Vec::new();
        let result = subagent_instruction_delivery::drain_with_after_registry_read(
            &child_id,
            &mut context,
            move || async move {
                let _ = read_tx.send(());
            },
        )
        .await;
        (result, context)
    });
    read_rx.await.expect("old execution captured");
    subagent_registry::unregister(&child.id).await;
    let new = register(&parent.id, &child.id).await;
    assert_eq!(new.run_id, old.run_id);
    child.subagent_run_id = Some(new.run_id.clone());
    session_store::save(&child).await.expect("save new execution");
    drop(guard);

    let (result, context) = drain.await.expect("join drain");
    let saved = session_store::get(&child.id).await.expect("saved child");
    assert!(result.is_err());
    assert!(context.is_empty());
    assert_eq!(saved.subagent_queued_prompts, vec!["ancienne correction"]);
    cleanup(&parent.id, &child.id, &sibling.id).await;
}

#[tokio::test]
async fn queued_prompt_save_rejects_replaced_execution() {
    let (parent, mut child, sibling) = sessions("enqueue race").await;
    let old = register(&parent.id, &child.id).await;
    let _sibling = register(&parent.id, &sibling.id).await;
    child.subagent_run_id = Some(old.run_id.clone());
    session_store::save(&child).await.expect("save old execution");
    let mut stale = child.clone();
    stale.subagent_queued_prompts.push("message obsolète".into());
    subagent_registry::unregister(&child.id).await;
    let new = register(&parent.id, &child.id).await;
    assert_eq!(new.run_id, old.run_id);

    let result = subagent_registry::save_queued_prompt_for_execution(
        &stale,
        &old.execution_id,
    )
    .await;
    let saved = session_store::get(&child.id).await.expect("saved child");
    assert!(result.is_err());
    assert!(saved.subagent_queued_prompts.is_empty());
    cleanup(&parent.id, &child.id, &sibling.id).await;
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
    let child = create_child(&parent.id, "Child").await;
    let sibling = create_child(&parent.id, "Sibling").await;
    (parent, child, sibling)
}

async fn create_child(parent_id: &str, name: &str) -> super::types_session::AgentSession {
    let mut child = session_store::create_full(name, "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent_id.into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    session_store::save(&child).await.expect("save child");
    child
}

async fn register(parent_id: &str, child_id: &str) -> subagent_registry::RegisteredSubagent {
    subagent_registry::register_execution(parent_id, child_id, CancellationToken::new())
        .await
        .expect("register execution")
}

async fn cleanup(parent_id: &str, child_id: &str, sibling_id: &str) {
    subagent_registry::unregister(child_id).await;
    subagent_registry::unregister(sibling_id).await;
    for id in [child_id, sibling_id, parent_id] {
        session_store::delete_one(id).await.expect("delete session");
    }
}
