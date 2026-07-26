use super::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};
use super::{session_store, subagent_context, subagent_history, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn empty_tool_result_survives_persistence_and_reconstruction() {
    let (parent, child, run) = active_child().await;
    let messages = vec![
        ChatMessage {
            role: "assistant".into(),
            content: "appel".into(),
            tool_calls: Some(vec![ToolCallOllama {
                id: Some("call-empty".into()),
                extra_content: None,
                function: ToolCallFunction {
                    name: "write_file".into(),
                    arguments: serde_json::json!({"path": "note.txt", "content": "ok"}),
                },
            }]),
            ..Default::default()
        },
        ChatMessage {
            role: "tool".into(),
            content: String::new(),
            tool_name: Some("write_file".into()),
            tool_call_id: Some("call-empty".into()),
            ..Default::default()
        },
    ];

    subagent_history::persist_for_execution(
        &child.id,
        &run.run_id,
        &run.execution_id,
        &messages,
    )
    .await
    .expect("persist history");
    let saved = session_store::get(&child.id).await.expect("load saved history");
    let rebuilt = subagent_context::build_messages(
        &child.id,
        "système".into(),
        "fallback",
        None,
    )
    .await;
    cleanup(&parent.id, &child.id).await;

    assert_eq!(saved.messages.len(), 2);
    assert_eq!(saved.messages[0].role, "assistant");
    assert_eq!(saved.messages[1].role, "tool");
    assert!(saved.messages[1].content.is_empty());
    assert!(saved.messages.len() <= 2_000);
    assert_eq!(rebuilt.len(), 3);
    assert_eq!(rebuilt[1].role, "assistant");
    assert_eq!(rebuilt[2].role, "tool");
    assert!(rebuilt[2].content.is_empty());
}

async fn active_child() -> (
    super::types_session::AgentSession,
    super::types_session::AgentSession,
    subagent_registry::RegisteredSubagent,
) {
    let parent = session_store::create_full("Parent empty tool", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Child empty tool", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("coder".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    let run = subagent_registry::register_execution(
        &parent.id,
        &child.id,
        CancellationToken::new(),
    )
    .await
    .expect("register child");
    child.subagent_run_id = Some(run.run_id.clone());
    session_store::save(&child).await.expect("save child");
    (parent, child, run)
}

async fn cleanup(parent_id: &str, child_id: &str) {
    subagent_registry::unregister(child_id).await;
    session_store::delete_one(child_id).await.expect("delete child");
    session_store::delete_one(parent_id).await.expect("delete parent");
}
