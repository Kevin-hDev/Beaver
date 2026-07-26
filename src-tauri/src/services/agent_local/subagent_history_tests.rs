use super::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};
use super::{session_store, subagent_history, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn history_persists_supported_roles_without_dropping_the_queue() {
    let (parent, mut child, registered) = active_child("history").await;
    child.subagent_queued_prompts.push("correction concurrente".into());
    session_store::save(&child).await.expect("save queue");
    let messages = vec![
        chat("system", "système"),
        chat("user", "mission"),
        ChatMessage {
            role: "assistant".into(),
            content: "appel".into(),
            tool_calls: Some(vec![ToolCallOllama {
                id: Some("call-42".into()),
                extra_content: None,
                function: ToolCallFunction {
                    name: "read_file".into(),
                    arguments: serde_json::json!({"path": "README.md"}),
                },
            }]),
            reasoning_content: Some("raisonnement".into()),
            ..Default::default()
        },
        ChatMessage {
            role: "tool".into(),
            content: "contenu".into(),
            tool_name: Some("read_file".into()),
            tool_call_id: Some("call-42".into()),
            ..Default::default()
        },
    ];

    let persisted = subagent_history::persist_for_execution(
        &child.id,
        &registered.run_id,
        &registered.execution_id,
        &messages,
    )
    .await
    .expect("persist history");

    let saved = session_store::get(&child.id).await.expect("load history");
    assert!(persisted);
    assert_eq!(saved.messages.len(), 3);
    assert_eq!(saved.messages[0].role, "user");
    assert_eq!(saved.messages[1].role, "assistant");
    assert_eq!(saved.messages[2].role, "tool");
    assert_eq!(saved.messages[1].thinking.as_deref(), Some("raisonnement"));
    assert_eq!(saved.messages[1].tool_calls.as_ref().map(Vec::len), Some(1));
    assert_eq!(saved.subagent_queued_prompts, vec!["correction concurrente"]);
    assert_eq!(
        saved.accumulated_tokens,
        crate::services::token_counting::estimate_agent_messages_tokens(&saved.messages)
    );
    cleanup(&parent.id, &child.id).await;
}

#[tokio::test]
async fn history_keeps_only_the_latest_two_thousand_messages() {
    let (parent, child, registered) = active_child("bounded").await;
    let messages = (0..2_002)
        .map(|index| chat("user", &format!("message-{index}")))
        .collect::<Vec<_>>();

    subagent_history::persist_for_execution(
        &child.id,
        &registered.run_id,
        &registered.execution_id,
        &messages,
    )
    .await
    .expect("persist bounded history");

    let saved = session_store::get(&child.id).await.expect("load bounded history");
    assert_eq!(saved.messages.len(), 2_000);
    assert_eq!(saved.messages[0].content, "message-2");
    assert_eq!(saved.messages[1_999].content, "message-2001");
    cleanup(&parent.id, &child.id).await;
}

#[tokio::test]
async fn history_save_and_concurrent_correction_both_survive() {
    let (parent, child, registered) = active_child("concurrent").await;
    let (loaded_tx, loaded_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let history_child = child.id.clone();
    let run_id = registered.run_id.clone();
    let execution_id = registered.execution_id.clone();
    let history = tokio::spawn(async move {
        subagent_history::persist_with_before_save(
            &history_child,
            &run_id,
            &execution_id,
            &[chat("user", "mission"), chat("assistant", "résultat")],
            move || async move {
                let _ = loaded_tx.send(());
                let _ = release_rx.await;
            },
        )
        .await
    });
    loaded_rx.await.expect("history loaded child");
    let correction_child = child.id.clone();
    let correction = tokio::spawn(async move {
        super::session_store_messages::add_redeployment_prompt(
            &correction_child,
            "correction concurrente",
        )
        .await
    });
    let mut correction = Box::pin(correction);
    assert!(tokio::time::timeout(std::time::Duration::from_millis(30), &mut correction)
        .await
        .is_err());

    let _ = release_tx.send(());
    history
        .await
        .expect("join history")
        .expect("persist history");
    correction
        .await
        .expect("join correction")
        .expect("persist correction");
    let saved = session_store::get(&child.id).await.expect("load concurrent child");
    assert_eq!(saved.messages.len(), 2);
    assert_eq!(saved.subagent_queued_prompts, vec!["correction concurrente"]);
    cleanup(&parent.id, &child.id).await;
}

fn chat(role: &str, content: &str) -> ChatMessage {
    ChatMessage {
        role: role.into(),
        content: content.into(),
        ..Default::default()
    }
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
