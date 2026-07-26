use super::{session_store, subagent_completion, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn terminal_event_boundary_precedes_registry_release_and_redeployment() {
    let parent = session_store::create_full("Parent event", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    let registered = subagent_registry::register_execution(
        &parent.id,
        &child.id,
        CancellationToken::new(),
    )
    .await
    .expect("register old execution");
    child.subagent_run_id = Some(registered.run_id.clone());
    session_store::save(&child).await.expect("save child");

    let (event_tx, event_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let completion_parent = parent.id.clone();
    let completion_child = child.id.clone();
    let callback_parent = completion_parent.clone();
    let callback_child = completion_child.clone();
    let old_run = registered.run_id.clone();
    let old_execution = registered.execution_id.clone();
    let completion = tokio::spawn(async move {
        subagent_completion::persist_terminal_completion_inner(
            &completion_parent,
            &completion_child,
            "explorer",
            subagent_status::COMPLETED,
            "Rapport terminal",
            true,
            Some((&old_run, &old_execution)),
            || async {},
            move |_outcome| async move {
                let active = subagent_registry::active_children_for_parent(&callback_parent)
                    .await
                    .contains(&callback_child);
                let _ = event_tx.send(active);
                let _ = release_rx.await;
            },
        )
        .await
    });

    assert!(event_rx.await.expect("terminal callback"));
    let restart_parent = parent.id.clone();
    let restart_child = child.id.clone();
    let restart = tokio::spawn(async move {
        let run_id = subagent_registry::get_or_create_run_id(&restart_parent).await;
        super::tool_delegate_child::prepare_existing_child(
            &restart_child,
            &restart_parent,
            "explorer",
            "Nouvelle mission",
            "Geminitor",
            "Reprise",
            "geminitor",
            &run_id,
        )
        .await
        .expect("prepare restart");
        super::tool_delegate_child::persist_delegate_prompt(
            &restart_child,
            "Nouvelle mission",
            true,
        )
        .await
        .expect("persist restart");
        subagent_registry::register_execution(
            &restart_parent,
            &restart_child,
            CancellationToken::new(),
        )
        .await
        .expect("register restart")
    });
    let mut restart = Box::pin(restart);
    assert!(tokio::time::timeout(std::time::Duration::from_millis(30), &mut restart)
        .await
        .is_err());

    let _ = release_tx.send(());
    completion
        .await
        .expect("join completion")
        .expect("complete old execution");
    let new_execution = restart.await.expect("join restart");
    assert!(subagent_registry::owns_execution(
        &child.id,
        &new_execution.run_id,
        &new_execution.execution_id,
    )
    .await);

    subagent_registry::unregister(&child.id).await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}
