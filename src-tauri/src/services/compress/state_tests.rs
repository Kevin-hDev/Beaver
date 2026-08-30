use super::state;
use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};
use crate::services::agent_local::types_session::AgentMessage;

fn chat(role: &str, content: &str) -> ChatMessage {
    match role {
        "system" => ChatMessage::system(content.to_string()),
        "user" => ChatMessage::user(content.to_string()),
        "assistant" => ChatMessage::assistant(content.to_string(), None, None, None, None),
        "tool" => ChatMessage::tool(content.to_string(), None, None),
        other => panic!("unsupported chat role in test/setup: {other}"),
    }
}

fn agent(role: &str, content: &str) -> AgentMessage {
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        turn_id: AgentMessage::new_turn_id(),
        role: role.to_string(),
        content: content.to_string(),
        message_kind: None,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        replay_source: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: None,
        stream_part: None,
    }
}

fn complete_turn(turn_id: &str, user: &str, assistant: &str) -> Vec<AgentMessage> {
    let mut user_message = agent("user", user);
    let mut assistant_message = agent("assistant", assistant);
    user_message.turn_id = turn_id.to_string();
    assistant_message.turn_id = turn_id.to_string();
    vec![user_message, assistant_message]
}

#[test]
fn context_used_prefers_larger_real_or_estimate() {
    assert_eq!(state::context_used_for_compression(Some(10), 12), 12);
    assert_eq!(state::context_used_for_compression(Some(15), 12), 15);
    assert_eq!(state::context_used_for_compression(None, 12), 12);
}

#[test]
fn request_start_index_uses_the_structured_runtime_barrier() {
    let mut current = chat("user", "vraie demande");
    current.continuity_barrier_before = true;
    let messages = vec![
        chat(
            "user",
            "This session is being continued from a previous conversation",
        ),
        current,
        chat("user", "/compress"),
    ];

    assert_eq!(state::request_start_index(&messages), 1);
}

#[test]
fn request_start_index_is_empty_after_a_terminal_compression_boundary() {
    let mut boundary = chat("assistant", "boundary");
    boundary.continuity_barrier_before = true;
    let messages = vec![boundary, chat("user", "/compress")];

    assert_eq!(state::request_start_index(&messages), messages.len());
}

#[test]
fn open_tool_chain_is_not_safe_to_compress() {
    let mut assistant = chat("assistant", "");
    assistant.tool_calls = Some(vec![ToolCallOllama {
        id: Some("call-1".to_string()),
        extra_content: None,
        function: ToolCallFunction {
            name: "read_file".to_string(),
            arguments: serde_json::json!({ "path": "a.rs" }),
        },
    }]);

    assert!(!state::is_safe_to_compress(&[assistant.clone()]));
    assert!(state::is_safe_to_compress(&[
        assistant,
        ChatMessage::tool(
            "ok".to_string(),
            Some("call-1".to_string()),
            Some("read_file".to_string())
        ),
    ]));
}

#[tokio::test]
async fn apply_and_save_keeps_the_two_recent_complete_turns() {
    let mut session = crate::services::agent_local::session_store::create_full(
        "Compression recent turns",
        "model",
        "openai",
        false,
        None,
    )
    .await
    .unwrap();
    session.messages = [
        complete_turn("turn-1", "u1", "a1"),
        complete_turn("turn-2", "u2", "a2"),
        complete_turn("turn-3", "u3", "a3"),
    ]
    .concat();
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();
    let mut runtime = vec![chat("user", "u3"), chat("assistant", "a3")];
    let working = tempfile::tempdir().unwrap();

    state::apply_and_save(
        &session.id,
        &mut runtime,
        "summary",
        16_000,
        false,
        working.path(),
        state::CompressionMode::Manual,
    )
    .await
    .unwrap();

    let reloaded = crate::services::agent_local::session_store::get(&session.id)
        .await
        .unwrap();
    assert_eq!(reloaded.compression_count, 1);
    assert_eq!(
        reloaded.messages[0].message_kind,
        Some(crate::services::agent_local::types_message::AgentMessageKind::CompressionCheckpoint)
    );
    assert_eq!(
        reloaded.messages[1].message_kind,
        Some(crate::services::agent_local::types_message::AgentMessageKind::CompressionBoundary)
    );
    let tail = reloaded
        .messages
        .iter()
        .rev()
        .take(4)
        .map(|message| message.content.as_str())
        .collect::<Vec<_>>();
    assert_eq!(tail, vec!["a3", "u3", "a2", "u2"]);
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn compression_keeps_a_checkpoint_available_for_commit() {
    let mut session = crate::services::agent_local::session_store::create_full(
        "Compression open journal",
        "model",
        "openai",
        false,
        None,
    )
    .await
    .unwrap();
    let turn_id = uuid::Uuid::new_v4().to_string();
    let user_id = uuid::Uuid::new_v4().to_string();
    let assistant_id = uuid::Uuid::new_v4().to_string();
    let request_id = uuid::Uuid::new_v4().to_string();
    let mut user = agent("user", "current question");
    user.id = user_id.clone();
    user.turn_id = turn_id.clone();
    session.messages = complete_turn("older-turn", "older question", "older answer");
    session.messages.push(user);
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();
    let mut journal = crate::services::agent_local::conversation_journal::ConversationJournal::new(
        session.id.clone(),
        turn_id,
        user_id,
        assistant_id,
        request_id,
    )
    .unwrap();
    journal
        .persist_assistant_step(&chat("assistant", "current answer"))
        .await
        .unwrap();
    let mut runtime = vec![
        chat("user", "current question"),
        chat("assistant", "current answer"),
    ];
    let working = tempfile::tempdir().unwrap();

    state::apply_and_save(
        &session.id,
        &mut runtime,
        "summary",
        16_000,
        false,
        working.path(),
        state::CompressionMode::Auto {
            request_start_index: 0,
        },
    )
    .await
    .unwrap();

    journal.commit_turn().await.unwrap();
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}
