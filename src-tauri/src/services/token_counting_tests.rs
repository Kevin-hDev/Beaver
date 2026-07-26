use super::*;

fn msg(content: &str) -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content: content.to_string(),
        ..Default::default()
    }
}

#[test]
fn estimates_ascii_like_previous_ratio() {
    assert_eq!(estimate_chat_tokens(&[msg(&"a".repeat(400))]), 100);
}

#[test]
fn estimates_accents_as_non_ascii_units() {
    assert_eq!(estimate_chat_tokens(&[msg("éé")]), 1);
}

#[test]
fn estimates_cjk_more_conservatively() {
    assert_eq!(estimate_chat_tokens(&[msg(&"你".repeat(1000))]), 1250);
    assert_eq!(estimate_chat_tokens(&[msg(&"こ".repeat(1000))]), 1250);
    assert_eq!(estimate_chat_tokens(&[msg(&"한".repeat(1000))]), 1250);
}

#[test]
fn estimates_emoji_as_wide() {
    assert_eq!(estimate_chat_tokens(&[msg("🎉")]), 2);
}

#[test]
fn sums_real_counts_only_when_both_present() {
    assert_eq!(sum_real_counts(Some(3), Some(4)), Some(7));
    assert_eq!(sum_real_counts(Some(3), None), None);
}

#[test]
fn agent_estimate_counts_tool_payload_once() {
    use crate::services::agent_local::types_session::{AgentMessage, ToolActivityRecord};

    let content = "a".repeat(400);
    let args = serde_json::json!({
        "path": "/memory/preference.md",
        "content": content,
    });
    let activity = ToolActivityRecord {
        name: "write_file".into(),
        summary: "/memory/preference.md".into(),
        domain: Some("memory".into()),
        resolved_path: None,
        args: Some(args.clone()),
        result: Some("ok".into()),
        is_error: None,
        content: Some(content),
        old_text: None,
        new_text: None,
        start_line: None,
        affected_paths: vec![],
        file_changes: vec![],
    };
    let message = AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "assistant".into(),
        content: String::new(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: Some(vec![activity]),
        segments: None,
        files: vec![],
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    };
    let expected = token_count_from_units(
        text_units("write_file") + text_units(&args.to_string()) + text_units("ok"),
    );

    assert_eq!(estimate_agent_message_tokens(&message), expected);
}
