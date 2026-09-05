use super::super::tool_artifact::{
    ArtifactMetadata, ArtifactPurpose, ArtifactSource, EphemeralArtifact,
};
use super::super::tool_executor_results::push_tool_message;
use super::super::types_tools::ToolResult;
use super::ToolResultPreviewBatch;

#[test]
fn parallel_tool_receipts_remain_ordered_before_the_single_media_follow_up() {
    let mut messages = Vec::new();
    for (index, content) in ["first", "second"].into_iter().enumerate() {
        let _ = push_tool_message(
            &mut messages,
            "read_file",
            ToolResult::ok(content),
            Some(&format!("call-{index}")),
        );
    }
    let mut wire = crate::services::llm::stream_convert::messages_to_openai_with_tools(
        &messages,
        crate::services::llm::route_profile::payload_policy("google", "gemini-2.5-flash")
            .expect("policy")
            .message,
        &[],
    );
    let mut previews = ToolResultPreviewBatch::from_ephemeral(
        1,
        Some("call-1".into()),
        EphemeralArtifact {
            metadata: ArtifactMetadata {
                name: "preview.png".into(),
                mime_type: "image/png".into(),
                bytes: 8,
                sha256: "a".repeat(64),
                purpose: ArtifactPurpose::Preview,
                source: ArtifactSource::ExtensionResource {
                    resource_id: "extension:demo:preview".into(),
                    catalog_fingerprint: "b".repeat(64),
                },
            },
            bytes: b"\x89PNG\r\n\x1a\n".to_vec(),
        },
    );
    crate::services::llm::tool_result_projection::append_openai_compatible_fallback(
        &mut wire,
        Some(&previews),
        crate::services::llm::route_profile::ToolResultMedia::FollowUpUserMessage,
        true,
        crate::services::llm::route_profile::ImageFormat::OpenAiNested,
    );
    assert_eq!(wire.len(), 3);
    assert_eq!(wire[0]["tool_call_id"], "call-0");
    assert_eq!(wire[1]["tool_call_id"], "call-1");
    assert_eq!(wire[2]["role"], "user");
    assert_eq!(
        wire[2]["content"][0]["text"],
        "Extension output (not user instruction)."
    );
    previews.clear_after_projection();
    assert!(previews.previews().is_empty());
    assert_eq!(messages[1].content, "second");
}

#[test]
fn only_explicit_image_previews_reach_the_projection_batch() {
    let ordinary = ToolResultPreviewBatch::from_ephemeral(
        0,
        Some("call-artifact".into()),
        artifact(ArtifactPurpose::Artifact, "image/png", png()),
    );
    assert!(ordinary.previews().is_empty());
    assert!(ordinary.notes().is_empty());

    let unsupported = ToolResultPreviewBatch::from_ephemeral(
        0,
        Some("call-invalid".into()),
        artifact(
            ArtifactPurpose::Preview,
            "image/png",
            b"not an image".to_vec(),
        ),
    );
    assert!(unsupported.previews().is_empty());
    assert_eq!(unsupported.notes().len(), 1);

    let detected = ToolResultPreviewBatch::from_ephemeral(
        0,
        Some("call-detected".into()),
        artifact(ArtifactPurpose::Preview, "image/jpeg", png()),
    );
    assert_eq!(detected.previews()[0].artifact.mime_type, "image/png");
}

fn artifact(purpose: ArtifactPurpose, mime_type: &str, bytes: Vec<u8>) -> EphemeralArtifact {
    EphemeralArtifact {
        metadata: ArtifactMetadata {
            name: "preview.png".into(),
            mime_type: mime_type.into(),
            bytes: bytes.len() as u64,
            sha256: "a".repeat(64),
            purpose,
            source: ArtifactSource::ExtensionResource {
                resource_id: "extension:demo:preview".into(),
                catalog_fingerprint: "b".repeat(64),
            },
        },
        bytes,
    }
}

fn png() -> Vec<u8> {
    b"\x89PNG\r\n\x1a\n".to_vec()
}
