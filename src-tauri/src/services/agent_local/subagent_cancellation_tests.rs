use super::{session_store, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn stale_cancel_cannot_touch_replacement_with_same_parent_run() {
    let parent = session("Parent", None).await;
    let mut child = session("Child", Some(&parent.id)).await;
    let sibling = session("Sibling", Some(&parent.id)).await;
    let old = register(&parent.id, &child.id).await;
    let _sibling = register(&parent.id, &sibling.id).await;
    child.subagent_run_id = Some(old.run_id.clone());
    session_store::save(&child).await.expect("save old run");
    subagent_registry::unregister(&child.id).await;
    let replacement = register(&parent.id, &child.id).await;
    child.subagent_run_id = Some(replacement.run_id.clone());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    session_store::save(&child).await.expect("save replacement");

    assert_eq!(old.run_id, replacement.run_id);
    assert!(!subagent_registry::cancel_execution(&child.id, &old.execution_id).await);
    let active = subagent_registry::active_run_for_child(&child.id)
        .await
        .expect("replacement active");
    assert!(!active.cancelled);
    assert_eq!(active.execution_id, replacement.execution_id);
    assert_eq!(
        session_store::get(&child.id)
            .await
            .expect("load child")
            .subagent_status
            .as_deref(),
        Some(subagent_status::RUNNING)
    );

    cleanup(&[&child.id, &sibling.id, &parent.id]).await;
}

#[tokio::test]
async fn duplicate_registration_never_replaces_active_execution() {
    let parent = session("Parent duplicate", None).await;
    let child = session("Child duplicate", Some(&parent.id)).await;
    let first = register(&parent.id, &child.id).await;

    let duplicate = subagent_registry::register_execution(
        &parent.id,
        &child.id,
        CancellationToken::new(),
    )
    .await;

    assert!(duplicate.is_err());
    assert!(subagent_registry::owns_execution(
        &child.id,
        &first.run_id,
        &first.execution_id,
    )
    .await);
    cleanup(&[&child.id, &parent.id]).await;
}

#[tokio::test]
async fn stopped_parent_stream_rejects_late_registration_but_fresh_stream_succeeds() {
    let parent = session("Parent stop", None).await;
    let child = session("Child stop", Some(&parent.id)).await;
    let stopped_stream = CancellationToken::new();
    let old_run = subagent_registry::get_or_create_run_id(&parent.id).await;
    stopped_stream.cancel();

    let rejected = subagent_registry::register_execution_for_parent_stream(
        &parent.id,
        &child.id,
        CancellationToken::new(),
        None,
        &stopped_stream,
    )
    .await;

    assert!(rejected.is_err());
    assert!(subagent_registry::active_children_for_parent(&parent.id)
        .await
        .is_empty());
    subagent_registry::release_run_claim(&parent.id, &old_run).await;

    let fresh_stream = CancellationToken::new();
    let _fresh_run = subagent_registry::get_or_create_run_id(&parent.id).await;
    let registered = subagent_registry::register_execution_for_parent_stream(
        &parent.id,
        &child.id,
        CancellationToken::new(),
        None,
        &fresh_stream,
    )
    .await
    .expect("fresh stream registers child");
    subagent_registry::cancel_all_for_parent(&parent.id).await;
    let active = subagent_registry::active_run_for_child(&child.id)
        .await
        .expect("registered child");
    assert_eq!(active.execution_id, registered.execution_id);
    assert!(active.cancelled);

    cleanup(&[&child.id, &parent.id]).await;
}

#[test]
fn parent_stream_token_reaches_delegate_registration_on_every_control_path() {
    let batch = include_str!("tool_executor_delegate_batch.rs");
    let dispatcher = include_str!("tool_dispatcher.rs");
    let fallback = include_str!("tool_dispatcher_fallback.rs");
    let message = include_str!("tool_subagent_message.rs");
    let delegate = include_str!("tool_delegate.rs");

    assert!(batch.contains("tool_executor_delegate_launch::launch"));
    assert!(batch.contains("cancel.clone()"));
    assert!(dispatcher.contains("dispatch_delegate(args, session_id, cancel.clone())"));
    assert!(fallback.contains(
        "tool_subagent_control::dispatch(tool_name, args, session_id, cancel.clone())"
    ));
    assert!(message.contains("dispatch_delegate(&payload, parent_id, cancel)"));
    assert!(delegate.contains("register_execution_for_parent_stream"));
    assert!(delegate.contains("&parent_cancel"));
}

#[tokio::test]
async fn stale_parent_stop_does_not_cancel_a_new_stream_child() {
    let parent_id = uuid::Uuid::new_v4().to_string();
    let old_child = uuid::Uuid::new_v4().to_string();
    let new_child = uuid::Uuid::new_v4().to_string();
    let old_owner = CancellationToken::new();
    let new_owner = CancellationToken::new();
    subagent_registry::register_execution_for_parent_stream(
        &parent_id,
        &old_child,
        CancellationToken::new(),
        None,
        &old_owner,
    )
    .await
    .expect("register old child");
    subagent_registry::register_execution_for_parent_stream(
        &parent_id,
        &new_child,
        CancellationToken::new(),
        None,
        &new_owner,
    )
    .await
    .expect("register new child");
    old_owner.cancel();

    subagent_registry::cancel_stopped_parent_stream_children(&parent_id).await;

    assert!(subagent_registry::active_run_for_child(&old_child)
        .await
        .expect("old child")
        .cancelled);
    assert!(!subagent_registry::active_run_for_child(&new_child)
        .await
        .expect("new child")
        .cancelled);
    subagent_registry::unregister(&old_child).await;
    subagent_registry::unregister(&new_child).await;
}

async fn session(
    name: &str,
    parent_id: Option<&str>,
) -> super::types_session::AgentSession {
    let mut session = session_store::create_full(name, "llama3", "ollama", false, None)
        .await
        .expect("create session");
    session.parent_session_id = parent_id.map(ToString::to_string);
    if parent_id.is_some() {
        session.subagent_type = Some("explorer".into());
        session.subagent_status = Some(subagent_status::RUNNING.into());
    }
    session_store::save(&session).await.expect("save session");
    session
}

async fn register(parent_id: &str, child_id: &str) -> subagent_registry::RegisteredSubagent {
    subagent_registry::register_execution(parent_id, child_id, CancellationToken::new())
        .await
        .expect("register execution")
}

async fn cleanup(ids: &[&str]) {
    for id in ids {
        subagent_registry::unregister(id).await;
        session_store::delete_one(id).await.expect("delete session");
    }
}
