use crate::models::agent_turn_contract::{NewUserTurnInput, TurnAttachmentInput};
use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};
use crate::services::agent_local::types_session::AgentMessage;

fn assistant_file(path: &str) -> ChatMessage {
    ChatMessage::assistant(
        String::new(),
        None,
        None,
        None,
        Some(vec![ToolCallOllama {
            id: None,
            extra_content: None,
            function: ToolCallFunction {
                name: "read_file".into(),
                arguments: serde_json::json!({"path": path}),
            },
        }]),
    )
}

#[tokio::test]
async fn files_are_reread_deduplicated_and_unavailable_content_is_generic() {
    let root = tempfile::tempdir().unwrap();
    tokio::fs::write(root.path().join("live.rs"), "first")
        .await
        .unwrap();
    let tool = || ChatMessage::tool("cached".into(), None, None);
    let messages = vec![
        assistant_file("live.rs"),
        tool(),
        assistant_file("live.rs"),
        tool(),
        assistant_file("gone.rs"),
        tool(),
    ];
    tokio::fs::write(root.path().join("live.rs"), "modified")
        .await
        .unwrap();

    let files = super::super::checkpoint_files::collect(&messages, root.path(), 128_000).await;

    assert_eq!(files.len(), 2);
    assert_eq!(files[0].current_content, "modified");
    assert_eq!(
        files[1].current_content,
        "[file unavailable: deleted, binary, or unreadable]"
    );
    assert!(!files[1]
        .current_content
        .contains(root.path().to_string_lossy().as_ref()));
}

#[tokio::test]
async fn inaccessible_attachment_does_not_discard_a_later_image() {
    use base64::Engine;
    let missing = TurnAttachmentInput {
        name: "missing.txt".into(),
        path: "/private/unavailable/missing.txt".into(),
        mime_type: "text/plain".into(),
        size: 1,
        thumbnail: None,
        access_grant: Some("v1.invalid".into()),
    };
    let png = base64::engine::general_purpose::STANDARD
        .encode([0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0]);
    let image = TurnAttachmentInput {
        name: "kept.png".into(),
        path: String::new(),
        mime_type: "image/png".into(),
        size: 12,
        thumbnail: Some(format!("data:image/png;base64,{png}")),
        access_grant: None,
    };
    let resolved = crate::services::agent_local::conversation_input::resolve_persisted_with_key(
        NewUserTurnInput {
            content: "question".into(),
            files: vec![missing, image],
            skills: Vec::new(),
        },
        &[7; 32],
    )
    .await
    .unwrap();

    assert!(resolved
        .provider_content
        .contains("[attachment unavailable]"));
    assert!(!resolved.provider_content.contains("/private/unavailable"));
    assert_eq!(resolved.images.len(), 1);
    assert_eq!(resolved.files.len(), 2);
}

#[test]
fn image_selection_keeps_metadata_and_obeys_provider_limit() {
    let mut message = bare_message();
    message.files = (0..3)
        .map(
            |index| crate::services::agent_local::types_session::FileAttachment {
                name: format!("{index}.png"),
                path: String::new(),
                mime_type: "image/png".into(),
                size: 12,
                thumbnail: Some("data:image/png;base64,iVBORw0KGgoAAAAA".into()),
                access_grant: None,
            },
        )
        .collect();

    let images = super::super::checkpoint_attachments::collect_images(&[message], 128_000, 2);

    assert_eq!(images.len(), 2);
    assert!(images.iter().all(|image| image.file.thumbnail.is_some()));
    assert!(images
        .iter()
        .all(|image| image.provider_payload.starts_with("iVBOR")));
}

#[test]
fn critical_references_are_deduplicated_and_bounded() {
    let reference = |id: &str| super::super::checkpoint_references::CheckpointReference {
        kind: "file".into(),
        id: id.into(),
        label: format!("reference-{id}"),
    };
    let selected = super::super::checkpoint_references::collect(
        [
            reference("a"),
            reference("a"),
            reference("b"),
            reference("c"),
        ],
        2,
        100,
    );
    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0].id, "a");
    assert_eq!(selected[1].id, "b");
}

fn bare_message() -> AgentMessage {
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        turn_id: uuid::Uuid::new_v4().to_string(),
        role: "user".into(),
        content: "images".into(),
        message_kind: None,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        replay_source: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: None,
        stream_part: None,
    }
}
