use super::{session_store, subagent_completion, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[test]
fn queued_boundary_continues_same_future_without_spawn_renew_or_early_cleanup() {
    let task = include_str!("subagent_task.rs");
    let continuation = task
        .find("if finalized.queued_followup")
        .expect("queued terminal continuation");
    let cleanup = task
        .find("subagent_task_change::cleanup_execution")
        .expect("final worktree cleanup");

    assert!(task.contains("loop {"));
    assert!(task[continuation..].contains("continue;"));
    assert!(continuation < cleanup);
    assert!(task.contains("cancel.clone()"));
    assert!(!task.contains("QueuedSubagentRun"));
    assert!(!task.contains("spawn_next_if_present"));
    assert!(!task.contains("renew_child"));
}

#[test]
fn legacy_second_spawn_pipeline_is_removed() {
    let modules = include_str!("agent_local_modules_core.rs");
    let registry = include_str!("subagent_registry.rs");
    let completion = include_str!("subagent_completion.rs");

    assert!(!modules.contains("subagent_queued"));
    assert!(!registry.contains("renew_child"));
    assert!(!completion.contains("persist_unstarted_followup_failure"));
}

#[test]
fn cancelled_run_never_auto_resumes_a_queued_instruction() {
    assert!(!super::subagent_task::should_continue_same_run(
        subagent_status::CANCELLED,
        true,
    ));
    let stream = include_str!("subagent_task_stream.rs");
    let cancelled = stream.find("was_cancelled || e == \"Annulé\"").unwrap();
    let delivery_error = stream.find("is_delivery_error").unwrap();
    assert!(cancelled < delivery_error);
}

#[tokio::test]
async fn cancelled_registry_run_refuses_message_and_drain() {
    let parent = session_store::create_full("Parent stop", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    let token = CancellationToken::new();
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    child.subagent_queued_prompts.push("avant stop".into());
    session_store::save(&child).await.expect("save child");
    let run_id = subagent_registry::register(&parent.id, &child.id, token.clone())
        .await
        .expect("register child");
    child.subagent_run_id = Some(run_id);
    session_store::save(&child).await.expect("save run id");
    token.cancel();

    let result = super::tool_subagent_message::run(
        &serde_json::json!({"subagent_id": child.id, "prompt": "après stop"}),
        &parent.id,
    )
    .await;
    let mut context = Vec::new();
    let drained = super::subagent_instruction_delivery::drain(&child.id, &mut context).await;
    let saved = session_store::get(&child.id).await.expect("saved child");

    subagent_registry::unregister(&child.id).await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
    assert!(result.is_error);
    assert!(result.content.contains("delegate_task"));
    assert!(result.content.contains("subagent_id"));
    assert!(drained.is_err());
    assert!(context.is_empty());
    assert_eq!(saved.subagent_queued_prompts, vec!["avant stop"]);
}

#[tokio::test]
async fn queued_boundary_keeps_registry_run_token_and_worktree_unchanged() {
    let parent = session_store::create_full("Parent boundary", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    let original_token = CancellationToken::new();
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("coder".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    child.subagent_worktree = Some("/tmp/existing-worktree".into());
    session_store::save(&child).await.expect("save child");
    let run_id = subagent_registry::register(
        &parent.id,
        &child.id,
        original_token.clone(),
    )
    .await
    .expect("register child");
    child.subagent_run_id = Some(run_id.clone());
    session_store::save(&child).await.expect("save run id");

    let child_lock = session_store::lock_session(&child.id).await;
    let guard = child_lock.lock().await;
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let parent_id = parent.id.clone();
    let child_id = child.id.clone();
    let completion = tokio::spawn(async move {
        started_tx.send(()).expect("signal terminal attempt");
        subagent_completion::persist_terminal_completion(
            &parent_id,
            &child_id,
            "coder",
            subagent_status::COMPLETED,
            "premier résultat",
        )
        .await
    });
    started_rx.await.expect("terminal attempt started");
    let mut queued_child = session_store::get(&child.id).await.expect("load child under lock");
    super::subagent_instruction_delivery::enqueue(&mut queued_child, "correction frontière")
        .expect("enqueue boundary correction");
    session_store::save(&queued_child).await.expect("save boundary correction");
    drop(guard);
    let finalized = completion
        .await
        .expect("join terminal completion")
        .expect("keep current run");
    let saved = session_store::get(&child.id).await.expect("saved child");
    let active_run = subagent_registry::get_run_id_for_child(&child.id).await;
    let cancelled = subagent_registry::cancel_one(&child.id).await;
    let reports = super::subagent_hidden_reports::peek_reports(&parent.id).await;

    subagent_registry::unregister(&child.id).await;
    session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
    assert!(finalized.queued_followup);
    assert_eq!(active_run.as_deref(), Some(run_id.as_str()));
    assert!(cancelled);
    assert!(original_token.is_cancelled());
    assert_eq!(saved.subagent_worktree, child.subagent_worktree);
    assert_eq!(saved.subagent_run_id.as_deref(), Some(run_id.as_str()));
    assert_eq!(saved.subagent_queued_prompts, vec!["correction frontière"]);
    assert!(reports.is_empty());
}
