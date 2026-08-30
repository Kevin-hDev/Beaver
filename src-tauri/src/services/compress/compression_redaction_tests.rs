use super::compression_redaction::{redact_checkpoint_text, redact_messages_for_compression};

#[test]
fn messages_and_checkpoint_are_redacted_without_mutating_the_source() {
    let session = super::snapshot_tests::session();
    let mut source = session.messages;
    source[0].content = "sk-proj-abcdefgh token=hunter2".to_string();
    source[1].thinking = Some("Bearer abcdefghijk".to_string());
    source[1].tool_activities = Some(vec![
        crate::services::agent_local::types_message::ToolActivityRecord {
            name: "bash".to_string(),
            summary: "secret output".to_string(),
            domain: None,
            resolved_path: None,
            args: Some(serde_json::json!({"token": "abcdefghijk"})),
            result: Some("-----BEGIN PRIVATE KEY-----\nsecret\n-----END PRIVATE KEY-----".into()),
            is_error: None,
            result_meta: None,
            content: None,
            old_text: None,
            new_text: None,
            start_line: None,
            affected_paths: Vec::new(),
            file_changes: Vec::new(),
        },
    ]);
    let original = serde_json::to_vec(&source).unwrap();

    let redacted = redact_messages_for_compression(&source);
    let payload = serde_json::to_string(&redacted).unwrap();

    assert!(!payload.contains("sk-proj-abcdefgh"));
    assert!(!payload.contains("hunter2"));
    assert!(!payload.contains("abcdefghijk"));
    assert!(!payload.contains("PRIVATE KEY"));
    assert_eq!(serde_json::to_vec(&source).unwrap(), original);
    let checkpoint = redact_checkpoint_text("Bearer abcdefghijk token=hunter2");
    assert!(!checkpoint.contains("abcdefghijk"));
    assert!(!checkpoint.contains("hunter2"));
}

#[test]
fn opaque_reasoning_envelope_is_left_byte_identical() {
    let mut source = super::snapshot_tests::session().messages;
    let before = source[1]
        .continuation
        .as_ref()
        .map(serde_json::to_vec)
        .transpose()
        .unwrap();

    let redacted = redact_messages_for_compression(&source);
    let after = redacted[1]
        .continuation
        .as_ref()
        .map(serde_json::to_vec)
        .transpose()
        .unwrap();

    assert_eq!(before, after);
    source.clear();
}
