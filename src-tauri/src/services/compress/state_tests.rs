use super::{state, state_recent};
use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};
use crate::services::agent_local::types_session::{
    AgentMessage, FileAttachment, ToolActivityRecord,
};

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
        role: role.to_string(),
        content: content.to_string(),
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
    }
}

#[test]
fn context_used_prefers_larger_real_or_estimate() {
    assert_eq!(state::context_used_for_compression(Some(10), 12), 12);
    assert_eq!(state::context_used_for_compression(Some(15), 12), 15);
    assert_eq!(state::context_used_for_compression(None, 12), 12);
}

#[test]
fn request_start_index_ignores_compression_context_and_command() {
    let messages = vec![
        chat("user", "vraie demande"),
        chat(
            "user",
            "This session is being continued from a previous conversation",
        ),
        chat(
            "user",
            "Recent file context preserved across compression:\n- a.rs",
        ),
        chat("user", "/compress"),
    ];

    assert_eq!(state::request_start_index(&messages), 0);
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

#[test]
fn keeps_two_recent_users_and_assistants() {
    let session = vec![
        agent("user", "u1"),
        agent("assistant", "a1"),
        agent("user", "u2"),
        agent("assistant", "a2"),
        agent("user", "u3"),
        agent("assistant", "a3"),
    ];

    let (_, recent) = state_recent::recent_messages(&session, &[]);
    let contents: Vec<_> = recent.iter().map(|m| m.content.as_str()).collect();

    assert_eq!(contents, vec!["u2", "a2", "u3", "a3"]);
}

#[test]
fn keeps_rich_fields_from_persisted_recent_messages() {
    let mut rich = agent("assistant", "answer");
    rich.thinking = Some("reasoning".to_string());
    rich.tool_activities = Some(vec![ToolActivityRecord {
        name: "bash".to_string(),
        summary: "ran tests".to_string(),
        domain: None,
        resolved_path: None,
        args: Some(serde_json::json!({ "cmd": "cargo test" })),
        result: Some("ok".to_string()),
        is_error: Some(false),
        result_meta: None,
        content: None,
        old_text: None,
        new_text: None,
        start_line: None,
        affected_paths: Vec::new(),
        file_changes: Vec::new(),
    }]);
    rich.files = vec![FileAttachment {
        name: "a.png".to_string(),
        path: "/tmp/a.png".to_string(),
        mime_type: "image/png".to_string(),
        size: 10,
        thumbnail: None,
        access_grant: None,
    }];
    rich.skill_names = Some(vec!["rust".to_string()]);

    let (_, recent) = state_recent::recent_messages(&[agent("user", "u"), rich], &[]);
    let saved = recent.iter().find(|m| m.role == "assistant").unwrap();

    assert_eq!(saved.thinking.as_deref(), Some("reasoning"));
    assert!(saved.tool_activities.is_some());
    assert_eq!(saved.files.len(), 1);
    assert_eq!(
        saved.skill_names.as_ref().unwrap(),
        &vec!["rust".to_string()]
    );
}

#[test]
fn runtime_assistant_not_yet_persisted_is_kept_for_session() {
    let session = vec![agent("user", "u1")];
    let runtime = vec![chat("user", "u1"), chat("assistant", "fresh answer")];

    let (_, recent) = state_recent::recent_messages(&session, &runtime);

    assert!(recent.iter().any(|m| m.content == "fresh answer"));
}
