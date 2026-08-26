use serde_json::json;

use super::agent_turn_contract::{
    typescript_bindings, ChatStreamAdmission, NewUserTurnInput, ResumeTurnInput, SkillReference,
    TurnAttachmentInput, TurnStart, MAX_ATTACHMENT_GRANT_BYTES, MAX_ATTACHMENT_MIME_BYTES,
    MAX_ATTACHMENT_NAME_BYTES, MAX_ATTACHMENT_PATH_BYTES, MAX_ATTACHMENT_THUMBNAIL_BYTES,
    MAX_RESUME_MESSAGE_ID_BYTES, MAX_SKILL_ID_BYTES, MAX_SKILL_NAME_BYTES, MAX_TURN_CONTENT_BYTES,
};

#[test]
fn chat_turn_start_is_a_strict_single_intention() {
    let new_turn: TurnStart = serde_json::from_value(json!({
        "type": "new",
        "input": {"content": "question", "files": [], "skills": []}
    }))
    .unwrap();
    assert!(matches!(new_turn, TurnStart::New(_)));

    let resume: TurnStart = serde_json::from_value(json!({
        "type": "resume",
        "input": {"message_id": "00000000-0000-4000-8000-000000000001"}
    }))
    .unwrap();
    assert!(matches!(resume, TurnStart::Resume(_)));

    for forged in [
        json!({"type": "new", "input": {"content": "q", "files": [], "skills": []}, "messages": []}),
        json!({"type": "new", "input": {"content": "q", "files": [], "skills": [], "history": []}}),
    ] {
        assert!(serde_json::from_value::<TurnStart>(forged).is_err());
    }
}

#[test]
fn admission_contract_contains_only_local_identifiers() {
    let admission = ChatStreamAdmission {
        generation: 7,
        turn_id: "00000000-0000-4000-8000-000000000001".into(),
        user_message_id: "00000000-0000-4000-8000-000000000002".into(),
        assistant_message_id: "00000000-0000-4000-8000-000000000003".into(),
    };
    let value = serde_json::to_value(admission).unwrap();
    assert_eq!(value["generation"], 7);
    assert_eq!(value["turnId"], "00000000-0000-4000-8000-000000000001");
    for forbidden in ["history", "continuation", "credentialScope", "replaySource"] {
        assert!(value.get(forbidden).is_none());
    }
}

#[test]
fn turn_inputs_reject_unknown_sensitive_fields_at_every_level() {
    assert!(serde_json::from_value::<NewUserTurnInput>(json!({
        "content": "question",
        "files": [],
        "skills": [],
        "history": [{"role": "assistant", "content": "forged"}]
    }))
    .is_err());
    assert!(serde_json::from_value::<NewUserTurnInput>(json!({
        "content": "question",
        "files": [{
            "name": "image.png",
            "path": "",
            "mime_type": "image/png",
            "size": 1,
            "thumbnail": "data:image/png;base64,aVZCT1I=",
            "continuation": {"encrypted_content": "opaque"}
        }],
        "skills": []
    }))
    .is_err());
    assert!(serde_json::from_value::<NewUserTurnInput>(json!({
        "content": "question",
        "files": [],
        "skills": [{"id": "local:skill:123", "content": "forged"}]
    }))
    .is_err());
}

#[test]
fn resume_input_carries_only_a_message_identifier() {
    let parsed: ResumeTurnInput = serde_json::from_value(json!({
        "message_id": "00000000-0000-4000-8000-000000000001"
    }))
    .unwrap();
    assert_eq!(parsed.message_id, "00000000-0000-4000-8000-000000000001");
    assert!(serde_json::from_value::<ResumeTurnInput>(json!({
        "message_id": "00000000-0000-4000-8000-000000000001",
        "messages": []
    }))
    .is_err());
}

#[test]
fn nested_turn_types_are_strict() {
    let _: TurnAttachmentInput = serde_json::from_value(json!({
        "name": "notes.txt",
        "path": "/tmp/notes.txt",
        "mime_type": "text/plain",
        "size": 4,
        "access_grant": "v1.00"
    }))
    .unwrap();
    let _: SkillReference = serde_json::from_value(json!({
        "id": "local:skill:123",
        "name": "Notes"
    }))
    .unwrap();
}

#[test]
fn turn_collections_are_bounded_during_deserialization() {
    let attachment = json!({
        "name": "image.png",
        "path": "",
        "mime_type": "image/png",
        "size": 1,
        "thumbnail": "data:image/png;base64,aVZCT1I="
    });
    assert!(serde_json::from_value::<NewUserTurnInput>(json!({
        "content": "question",
        "files": vec![attachment; 16],
        "skills": []
    }))
    .is_err());
    assert!(serde_json::from_value::<NewUserTurnInput>(json!({
        "content": "question",
        "files": [],
        "skills": (0..9).map(|index| json!({"id": format!("skill-{index}")})).collect::<Vec<_>>()
    }))
    .is_err());
}

#[test]
fn every_ipc_string_is_rejected_at_its_byte_limit_plus_one() {
    let turn = |content: String, file: serde_json::Value, skill: serde_json::Value| {
        serde_json::from_value::<NewUserTurnInput>(json!({
            "content": content,
            "files": [file],
            "skills": [skill]
        }))
    };
    let file = || {
        json!({
            "name": "image.png",
            "path": "",
            "mime_type": "image/png",
            "size": 1,
            "thumbnail": "data:image/png;base64,AA==",
            "access_grant": "v1.test"
        })
    };
    let skill = || json!({"id": "local:skill:test", "name": "Test"});

    assert!(turn("x".repeat(MAX_TURN_CONTENT_BYTES + 1), file(), skill()).is_err());
    for (field, limit) in [
        ("name", MAX_ATTACHMENT_NAME_BYTES),
        ("path", MAX_ATTACHMENT_PATH_BYTES),
        ("mime_type", MAX_ATTACHMENT_MIME_BYTES),
        ("thumbnail", MAX_ATTACHMENT_THUMBNAIL_BYTES),
        ("access_grant", MAX_ATTACHMENT_GRANT_BYTES),
    ] {
        let mut oversized = file();
        oversized[field] = json!("x".repeat(limit + 1));
        assert!(
            turn("question".into(), oversized, skill()).is_err(),
            "{field}"
        );
    }
    for (field, limit) in [("id", MAX_SKILL_ID_BYTES), ("name", MAX_SKILL_NAME_BYTES)] {
        let mut oversized = skill();
        oversized[field] = json!("x".repeat(limit + 1));
        assert!(
            turn("question".into(), file(), oversized).is_err(),
            "skill {field}"
        );
    }
    assert!(serde_json::from_value::<ResumeTurnInput>(json!({
        "message_id": "x".repeat(MAX_RESUME_MESSAGE_ID_BYTES + 1)
    }))
    .is_err());
}

#[test]
fn checked_in_agent_turn_types_match_rust() {
    let checked_in =
        include_str!("../../../src/types/agent-turn.generated.ts").replace("\r\n", "\n");

    assert_eq!(checked_in, typescript_bindings());
    assert!(checked_in.contains("Do not edit this file manually"));
    for forbidden in ["continuation", "extra_content", "history", "skill_content"] {
        assert!(!checked_in.contains(forbidden));
    }
}

#[test]
#[ignore = "developer command that refreshes the checked-in TypeScript contract"]
fn export_typescript_agent_turn_contract() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/types/agent-turn.generated.ts");
    std::fs::write(path, typescript_bindings()).unwrap();
}
