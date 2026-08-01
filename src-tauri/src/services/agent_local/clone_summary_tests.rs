use super::*;
use chrono::Utc;
use serde_json::json;

fn message_with_tool(tool: ToolActivityRecord) -> AgentMessage {
    AgentMessage {
        id: "m1".into(),
        role: "assistant".into(),
        content: "done".into(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: Some(vec![tool]),
        segments: None,
        files: vec![],
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    }
}

fn tool_record(name: &str, path: &str, result: Option<String>) -> ToolActivityRecord {
    ToolActivityRecord {
        name: name.into(),
        summary: if name == "edit_file" { "edit" } else { "read" }.into(),
        domain: None,
        resolved_path: None,
        args: Some(json!({"path": path})),
        result,
        is_error: None,
        result_meta: None,
        content: None,
        old_text: None,
        new_text: None,
        start_line: None,
        affected_paths: vec![],
        file_changes: vec![],
    }
}

#[test]
fn serialize_limits_tool_result() {
    let tool = tool_record(
        "read_file",
        "src/main.rs",
        Some("a".repeat(MAX_TOOL_RESULT_CHARS + 50)),
    );
    let serialized = serialize_messages(&[message_with_tool(tool)]);
    assert!(serialized.len() < MAX_TOOL_RESULT_CHARS + 500);
}

#[test]
fn extract_files_uses_tool_traces() {
    let mut tool = tool_record("edit_file", "src/lib.rs", None);
    tool.affected_paths.push("src/lib.rs".into());
    let (read, modified) = extract_traced_files(&[message_with_tool(tool)]);
    assert!(read.is_empty());
    assert_eq!(modified, vec!["src/lib.rs"]);
}

#[test]
fn serialize_adds_truncated_marker_at_limit() {
    let mut msg = message_with_tool(tool_record(
        "read_file",
        "src/main.rs",
        Some("ok".into()),
    ));
    msg.content = "a".repeat(MAX_SUMMARY_INPUT_CHARS);
    let serialized = serialize_messages(&[msg]);
    assert!(serialized.ends_with("[Truncated]"));
    assert!(serialized.chars().count() <= MAX_SUMMARY_INPUT_CHARS);
}

fn inherited_context_message(summary: &str) -> AgentMessage {
    AgentMessage {
        id: "inherited".into(),
        role: "user".into(),
        content: format!("{CLONE_SUMMARY_PREFIX}\n\n{summary}"),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    }
}

#[test]
fn serialize_messages_extracts_inherited_context() {
    // Un suffixe contenant un hidden context message (résumé parent) :
    // celui-ci doit être extrait dans une section <inherited_context> en tête,
    // pas sérialisé comme un message utilisateur normal.
    let inherited = inherited_context_message("Previous attempt hit a bug in parser.rs.");
    let normal = AgentMessage {
        id: "after".into(),
        role: "user".into(),
        content: "Now let's try a different approach.".into(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    };
    let serialized = serialize_messages(&[inherited, normal]);

    assert!(
        serialized.starts_with("<inherited_context>"),
        "doit commencer par <inherited_context>, obtenu: {serialized:?}"
    );
    assert!(serialized.contains("Previous attempt hit a bug in parser.rs."));
    assert!(serialized.contains("</inherited_context>"));
    // Le message normal est bien sérialisé après, dans une balise <message>.
    assert!(serialized.contains("<message role=\"user\">"));
    assert!(serialized.contains("Now let's try a different approach."));
}

#[test]
fn serialize_messages_extracts_inherited_context_anywhere_in_suffix() {
    // Même si le hidden context message apparaît au milieu du suffixe,
    // il doit être extrait vers <inherited_context> en tête (pas coupé du flux).
    let inherited = inherited_context_message("Avoid the off-by-one in loop.");
    let before = AgentMessage {
        id: "before".into(),
        role: "assistant".into(),
        content: "first attempt".into(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    };
    let after = AgentMessage {
        id: "after".into(),
        role: "user".into(),
        content: "retry".into(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    };
    let serialized = serialize_messages(&[before.clone(), inherited, after]);

    // Le bloc inherited_context est en tête...
    assert!(serialized.starts_with("<inherited_context>"));
    assert!(serialized.contains("Avoid the off-by-one in loop."));
    // ...et les deux messages normaux sont bien présents (ordre conservé).
    assert!(serialized.contains("first attempt"));
    assert!(serialized.contains("retry"));
    // Aucune balise <message> ne doit contenir le préfixe de contexte hérité :
    // le préfixe ne doit apparaître que dans la section <inherited_context>.
    let after_inherited = serialized
        .split("</inherited_context>")
        .nth(1)
        .unwrap_or("");
    assert!(
        !after_inherited.contains(CLONE_SUMMARY_PREFIX),
        "le préfixe hérité ne doit pas fuiter dans les <message>"
    );
}

#[test]
fn serialize_messages_omits_inherited_context_when_absent() {
    // Sans hidden context message, pas de bloc <inherited_context>.
    let normal = AgentMessage {
        id: "m1".into(),
        role: "user".into(),
        content: "hello".into(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    };
    let serialized = serialize_messages(&[normal]);
    assert!(!serialized.contains("<inherited_context>"));
    assert!(serialized.contains("<message role=\"user\">"));
}
