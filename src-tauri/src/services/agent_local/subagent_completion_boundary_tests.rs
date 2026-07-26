use super::{
    session_store, subagent_completion, subagent_registry, subagent_status,
    tool_delegate_child,
};
use super::types_session::AgentMessage;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn message_between_terminal_save_and_registry_completion_is_never_stranded() {
    let parent = session_store::create_full("Parent boundary", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    session_store::save(&child).await.expect("save child");
    let completed_run = subagent_registry::register(
        &parent.id,
        &child.id,
        CancellationToken::new(),
    )
        .await
        .expect("register child");

    let parent_lock = session_store::lock_session(&parent.id).await;
    let parent_guard = parent_lock.lock().await;
    let completion_parent = parent.id.clone();
    let completion_child = child.id.clone();
    let completion = tokio::spawn(async move {
        subagent_completion::persist_terminal_completion(
            &completion_parent,
            &completion_child,
            "explorer",
            subagent_status::COMPLETED,
            "Rapport terminal",
        )
        .await
    });
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            if session_store::get(&child.id)
                .await
                .ok()
                .and_then(|saved| saved.subagent_status)
                .as_deref()
                == Some(subagent_status::COMPLETED)
            {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("terminal status saved before report");

    let restart_parent = parent.id.clone();
    let restart_child = child.id.clone();
    let (attempt_tx, attempt_rx) = tokio::sync::oneshot::channel();
    let restart = tokio::spawn(async move {
        let child_lock = session_store::lock_session(&restart_child).await;
        let _ = attempt_tx.send(());
        let child_guard = child_lock.lock().await;
        drop(child_guard);
        let run_id = subagent_registry::get_or_create_run_id(&restart_parent).await;
        let prompt = "Nouvelle instruction à la frontière";
        tool_delegate_child::prepare_existing_child(
            &restart_child,
            &restart_parent,
            "explorer",
            prompt,
            "Geminitor",
            "Analyse",
            "geminitor",
            &run_id,
        )
        .await
        .expect("prepare existing child after boundary");
        session_store::add_messages(
            &restart_child,
            vec![AgentMessage {
                id: uuid::Uuid::new_v4().to_string(),
                role: "user".to_string(),
                content: prompt.to_string(),
                thinking: None,
                tool_calls: None,
                tool_name: None,
                tool_activities: None,
                segments: None,
                files: vec![],
                timestamp: chrono::Utc::now(),
                tokens: 0,
                work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
            }],
            0,
        )
        .await
        .expect("persist restart prompt");
        let registered_run = subagent_registry::register(
            &restart_parent,
            &restart_child,
            CancellationToken::new(),
        )
        .await
        .expect("register explicit restart");
        (registered_run, prompt.to_string())
    });
    attempt_rx.await.expect("restart attempts child boundary");
    drop(parent_guard);
    completion
        .await
        .expect("completion task")
        .expect("persist completion");
    let (restarted_run, accepted_prompt) = restart.await.expect("restart task");

    let saved = session_store::get(&child.id).await.expect("saved child");
    assert_ne!(completed_run, restarted_run);
    assert_eq!(saved.subagent_status.as_deref(), Some(subagent_status::RUNNING));
    assert_eq!(saved.subagent_prompt.as_deref(), Some(accepted_prompt.as_str()));
    assert!(saved.messages.iter().any(|message| {
        message.role == "user" && message.content == accepted_prompt
    }));
    assert!(subagent_registry::active_children_for_parent(&parent.id)
        .await
        .contains(&child.id));
    assert_eq!(
        super::subagent_hidden_reports::peek_reports(&parent.id).await.len(),
        1
    );
    let terminal = subagent_registry::terminal_state_for_parent(&parent.id)
        .await
        .expect("successful terminal remains pending");
    assert_eq!(terminal.sequence, 1);
    assert!(!terminal.report_persistence_failed);

    subagent_registry::unregister(&child.id).await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}
