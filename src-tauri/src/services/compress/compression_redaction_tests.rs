use super::compression_redaction::{redact_checkpoint_text, redact_messages_for_compression};

#[derive(serde::Deserialize)]
struct SerializationFailure;

impl serde::Serialize for SerializationFailure {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom("fixture serialization failure"))
    }
}

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
fn opaque_reasoning_envelope_is_excluded_only_from_the_summary_copy() {
    let mut source = super::snapshot_tests::session().messages;
    source[1].continuation = Some(
        crate::services::reasoning_continuity::envelope::ReasoningEnvelope::new(
            crate::services::reasoning_continuity::contract::ContractId::OllamaNativeV1,
            crate::services::reasoning_continuity::envelope::ReasoningSource {
                route_id: crate::services::reasoning_continuity::contract::RouteId::Ollama,
                model_id: "fixture".into(),
                credential_scope:
                    crate::services::reasoning_continuity::contract::CredentialScope::local_uncredentialed(),
                reasoning_mode:
                    crate::services::reasoning_continuity::contract::ReasoningModeId::High,
            },
            crate::services::reasoning_continuity::envelope::CompletionState::Complete,
            crate::services::reasoning_continuity::envelope::ContinuationState::OllamaNative {
                thinking: "token=hunter2".into(),
            },
            Vec::new(),
        ),
    );
    let before = source[1]
        .continuation
        .as_ref()
        .map(serde_json::to_vec)
        .transpose()
        .unwrap();

    let redacted = redact_messages_for_compression(&source);
    assert!(redacted[1].continuation.is_none());
    let source_after = source[1]
        .continuation
        .as_ref()
        .map(serde_json::to_vec)
        .transpose()
        .unwrap();
    assert_eq!(before, source_after);
    source.clear();
}

#[test]
fn structured_redaction_failure_drops_the_unfiltered_value() {
    let mut value = Some(vec![SerializationFailure]);
    super::compression_redaction::redact_serializable(&mut value);
    assert!(value.is_none());
}
