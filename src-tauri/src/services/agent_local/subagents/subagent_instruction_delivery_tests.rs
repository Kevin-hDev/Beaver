use super::{session_store, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn queued_instructions_are_drained_in_order_without_changing_run_id() {
    let (parent, mut child, run_id) = active_child().await;
    let queued = vec![
        "première correction".to_string(),
        "  deuxième\n  correction  ".to_string(),
    ];
    child.subagent_queued_prompts = queued.clone();
    session_store::save(&child).await.expect("save queue");
    let mut request_messages = Vec::new();

    let drained = super::subagent_instruction_delivery::drain(
        &child.id,
        &mut request_messages,
    )
    .await
    .expect("drain live instructions");
    let saved = session_store::get(&child.id).await.expect("saved child");

    cleanup(&parent.id, &child.id).await;
    assert_eq!(drained, 2);
    assert_eq!(saved.subagent_run_id.as_deref(), Some(run_id.as_str()));
    assert!(saved.subagent_queued_prompts.is_empty());
    assert_eq!(user_contents(&saved.messages), queued);
    assert_eq!(chat_user_contents(&request_messages), queued);
    assert_eq!(
        saved.accumulated_tokens,
        crate::services::token_counting::estimate_agent_messages_tokens(&saved.messages)
    );
}

#[tokio::test]
async fn run_id_mismatch_fails_closed_with_queue_and_context_untouched() {
    let (parent, mut child, _) = active_child().await;
    child.subagent_run_id = Some("stale-run".into());
    child.subagent_queued_prompts.push("correction".into());
    session_store::save(&child).await.expect("save stale run");
    let mut request_messages = Vec::new();

    let result = super::subagent_instruction_delivery::drain(
        &child.id,
        &mut request_messages,
    )
    .await;
    let saved = session_store::get(&child.id).await.expect("saved child");

    cleanup(&parent.id, &child.id).await;
    assert!(result.is_err());
    assert_eq!(saved.subagent_queued_prompts, vec!["correction"]);
    assert!(request_messages.is_empty());
}

#[tokio::test]
async fn save_failure_keeps_queue_and_terminalizes_without_registry_ghost() {
    let (parent, mut child, _) = active_child().await;
    child.subagent_queued_prompts.push("correction durable".into());
    session_store::save(&child).await.expect("save queue");
    let child_path = session_path(&child.id);
    let backup_path = child_path.with_extension("json.live-correction-backup");
    let sabotage_child = child_path.clone();
    let sabotage_backup = backup_path.clone();
    let mut request_messages = Vec::new();

    let result = super::subagent_instruction_delivery::drain_with_before_save(
        &child.id,
        &mut request_messages,
        move || async move {
            tokio::fs::rename(&sabotage_child, &sabotage_backup)
                .await
                .expect("backup child session");
            tokio::fs::create_dir(&sabotage_child)
                .await
                .expect("block atomic rename");
        },
    )
    .await;

    if child_path.is_dir() {
        tokio::fs::remove_dir(&child_path)
            .await
            .expect("remove save blocker");
    }
    if backup_path.exists() {
        tokio::fs::rename(&backup_path, &child_path)
            .await
            .expect("restore child session");
    }
    let active_run = subagent_registry::active_run_for_child(&child.id)
        .await
        .expect("active execution");
    let marked_after_failed_save = subagent_registry::prompt_was_delivered(
        &child.id,
        &active_run.execution_id,
        "correction durable",
    )
    .await;
    super::subagent_completion::persist_instruction_delivery_failure(
        &parent.id,
        &child.id,
        "coder",
    )
    .await
    .expect("terminalize delivery failure");
    let saved = session_store::get(&child.id).await.expect("restored child");
    let active = subagent_registry::active_children_for_parent(&parent.id).await;
    let reports = super::subagent_hidden_reports::peek_reports(&parent.id).await;
    let terminal = subagent_registry::terminal_state_for_parent(&parent.id)
        .await
        .expect("terminal state");
    let mut parent_messages = Vec::new();
    let mut orchestrator =
        super::subagent_orchestration::ParentSubagentOrchestrator::new(&parent.id).await;
    let injected = orchestrator.inject_pending_reports(&mut parent_messages).await;
    remove_failed_save_temps(&child.id).await;
    cleanup(&parent.id, &child.id).await;

    assert!(result.is_err());
    assert!(!marked_after_failed_save);
    assert_eq!(saved.subagent_queued_prompts, vec!["correction durable"]);
    assert_eq!(saved.subagent_status.as_deref(), Some(subagent_status::FAILED));
    assert!(active.is_empty());
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].status, subagent_status::FAILED);
    assert_eq!(terminal.sequence, 1);
    assert!(!terminal.report_persistence_failed);
    assert!(injected);
    assert!(request_messages.is_empty());
}

async fn active_child() -> (
    super::types_session::AgentSession,
    super::types_session::AgentSession,
    String,
) {
    let parent = session_store::create_full("Parent drain", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    session_store::save(&child).await.expect("save child");
    let run_id = subagent_registry::register(&parent.id, &child.id, CancellationToken::new())
        .await
        .expect("register child");
    child.subagent_run_id = Some(run_id.clone());
    session_store::save(&child).await.expect("save run id");
    (parent, child, run_id)
}

fn user_contents(messages: &[super::types_session::AgentMessage]) -> Vec<String> {
    messages
        .iter()
        .filter(|message| message.role == "user")
        .map(|message| message.content.clone())
        .collect()
}

fn chat_user_contents(messages: &[super::types_ollama::ChatMessage]) -> Vec<String> {
    messages
        .iter()
        .filter(|message| message.role == "user")
        .map(|message| message.content.clone())
        .collect()
}

async fn cleanup(parent_id: &str, child_id: &str) {
    subagent_registry::unregister(child_id).await;
    session_store::delete_one(child_id).await.expect("delete child");
    session_store::delete_one(parent_id)
        .await
        .expect("delete parent");
}

fn session_path(child_id: &str) -> std::path::PathBuf {
    crate::services::paths::data_dir()
        .join("agent-sessions")
        .join(format!("{child_id}.json"))
}

async fn remove_failed_save_temps(child_id: &str) {
    let directory = crate::services::paths::data_dir().join("agent-sessions");
    let prefix = format!(".{child_id}.");
    let mut entries = tokio::fs::read_dir(directory).await.expect("read sessions");
    while let Some(entry) = entries.next_entry().await.expect("read session entry") {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&prefix) && name.ends_with(".tmp") {
            tokio::fs::remove_file(entry.path())
                .await
                .expect("remove failed save temp");
        }
    }
}
