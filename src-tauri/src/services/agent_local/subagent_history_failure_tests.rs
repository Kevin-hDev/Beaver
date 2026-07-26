use super::types_ollama::ChatMessage;
use super::{session_store, subagent_history, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn history_save_failure_keeps_previous_history_and_queue_intact() {
    let (parent, mut child, registered) = active_child("save failure").await;
    child.messages.push(saved_message("ancienne histoire"));
    child.subagent_queued_prompts.push("correction durable".into());
    session_store::save(&child).await.expect("save initial child");
    let path = crate::services::paths::data_dir()
        .join("agent-sessions")
        .join(format!("{}.json", child.id));
    let backup = path.with_extension("json.history-backup");
    let blocked = path.clone();
    let backup_path = backup.clone();

    let result = subagent_history::persist_with_before_save(
        &child.id,
        &registered.run_id,
        &registered.execution_id,
        &[chat("user", "nouvelle histoire")],
        move || async move {
            tokio::fs::rename(&blocked, &backup_path)
                .await
                .expect("backup child");
            tokio::fs::create_dir(&blocked)
                .await
                .expect("block save");
        },
    )
    .await;
    tokio::fs::remove_dir(&path).await.expect("remove blocker");
    tokio::fs::rename(&backup, &path)
        .await
        .expect("restore child");
    let saved = session_store::get(&child.id).await.expect("load restored child");

    assert!(result.is_err());
    assert_eq!(saved.messages.len(), 1);
    assert_eq!(saved.messages[0].content, "ancienne histoire");
    assert_eq!(saved.subagent_queued_prompts, vec!["correction durable"]);
    cleanup(&parent.id, &child.id).await;
}

#[test]
fn task_persists_history_before_terminal_completion_and_fails_closed() {
    let source = include_str!("subagent_task.rs");
    let history = source.find("subagent_history::persist_for_execution").unwrap();
    let terminal = history
        + source[history..]
            .find("subagent_completion_events::persist_terminal")
            .unwrap();
    assert!(history < terminal);
    assert!(source[history..terminal].contains("Err(_)"));
    assert!(source[history..terminal].contains("subagent_status::FAILED"));
}

fn chat(role: &str, content: &str) -> ChatMessage {
    ChatMessage {
        role: role.into(),
        content: content.into(),
        ..Default::default()
    }
}

fn saved_message(content: &str) -> super::types_session::AgentMessage {
    super::subagent_instruction_delivery::agent_message(content)
}

async fn active_child(
    suffix: &str,
) -> (
    super::types_session::AgentSession,
    super::types_session::AgentSession,
    subagent_registry::RegisteredSubagent,
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
    let mut child = session_store::create_full("Child", "llama3", "ollama", false, None)
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
    .expect("register child");
    child.subagent_run_id = Some(registered.run_id.clone());
    session_store::save(&child).await.expect("save child");
    (parent, child, registered)
}

async fn cleanup(parent_id: &str, child_id: &str) {
    subagent_registry::unregister(child_id).await;
    session_store::delete_one(child_id).await.expect("delete child");
    session_store::delete_one(parent_id).await.expect("delete parent");
}
