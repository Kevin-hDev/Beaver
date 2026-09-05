#[test]
fn responses_keeps_call_id() {
    let value = super::responses_tool_output("call-1", "result");
    assert_eq!(value["call_id"], "call-1");
}

#[test]
fn responses_fixture_uses_documented_function_output_shape() {
    let expected: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../test-fixtures/tool-result-media/openai-responses.json"
    ))
    .unwrap();
    assert_eq!(
        super::responses_tool_output("call-fixture", "Extension output: preview available"),
        expected
    );
}

#[test]
fn anthropic_groups_text_before_images() {
    let value = super::anthropic_tool_result("call-1", "result", &[]);
    assert_eq!(value["content"][0]["type"], "text");
}

#[test]
fn fallback_keeps_tool_order_and_has_one_bounded_follow_up() {
    let batch = previews(9);
    assert_eq!(batch.omitted(), 1);
    let mut messages = vec![
        serde_json::json!({"role":"tool","content":"first","tool_call_id":"call-1"}),
        serde_json::json!({"role":"tool","content":"second","tool_call_id":"call-2"}),
    ];
    super::append_openai_compatible_fallback(
        &mut messages,
        Some(&batch),
        crate::services::llm::route_profile::ToolResultMedia::FollowUpUserMessage,
        true,
        crate::services::llm::route_profile::ImageFormat::OpenAiNested,
    );
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0]["content"], "first");
    assert_eq!(messages[1]["content"], "second");
    let content = messages[2]["content"].as_array().unwrap();
    assert_eq!(content.len(), 18);
    assert_eq!(
        content.last().unwrap()["text"],
        "Additional extension previews are available in Beaver."
    );
}

#[test]
fn parallel_preview_batch_keeps_first_eight_in_original_result_order() {
    let batch = crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch::from_ephemerals(
        (0..16).map(|index| {
            let tool_index = usize::from(index >= 8);
            (tool_index, Some(format!("call-{tool_index}")), artifact(index))
        }),
    );
    let ids = batch
        .previews()
        .iter()
        .map(|preview| preview.tool_call_id.as_deref())
        .collect::<Vec<_>>();

    assert_eq!(
        batch.previews().len(),
        crate::services::extensions::types::MAX_MULTIMODAL_PREVIEWS_PER_CONTINUATION
    );
    assert_eq!(batch.omitted(), 8);
    assert_eq!(batch.omitted_sources().len(), 1);
    assert_eq!(batch.omitted_sources()[0].tool_call_index, 1);
    assert_eq!(
        batch.omitted_sources()[0].tool_call_id.as_deref(),
        Some("call-1")
    );
    for id in ids {
        assert_eq!(id, Some("call-0"));
    }
}

#[test]
fn vision_and_tool_projection_share_the_generated_preview_limit() {
    assert_eq!(
        crate::services::llm::vision::MAX_IMAGES_PER_MESSAGE,
        crate::services::extensions::types::MAX_MULTIMODAL_PREVIEWS_PER_CONTINUATION
    );
    let vision_source = include_str!("vision.rs");
    assert!(vision_source
        .contains("crate::services::extensions::types::MAX_MULTIMODAL_PREVIEWS_PER_CONTINUATION"));
    assert!(!vision_source.contains("MAX_IMAGES_PER_MESSAGE: usize = 8"));
    assert!(!include_str!("tool_result_projection.rs")
        .contains("MAX_MULTIMODAL_PREVIEWS_PER_CONTINUATION: usize = 8"));
}

#[test]
fn fallback_refuses_bytes_without_vision_or_supported_wire() {
    let batch = previews(1);
    for (vision, format) in [
        (
            false,
            crate::services::llm::route_profile::ImageFormat::OpenAiNested,
        ),
        (
            true,
            crate::services::llm::route_profile::ImageFormat::Unsupported,
        ),
    ] {
        let mut messages = vec![serde_json::json!({"role":"tool","content":"tool"})];
        super::append_openai_compatible_fallback(
            &mut messages,
            Some(&batch),
            crate::services::llm::route_profile::ToolResultMedia::FollowUpUserMessage,
            vision,
            format,
        );
        assert_eq!(messages.len(), 1);
    }
}

#[test]
fn unsupported_wire_does_not_emit_a_text_only_follow_up_for_replay_notes() {
    let batch =
        crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch::from_ephemeral(
            0,
            Some("call-invalid".into()),
            invalid_preview(),
        );
    assert_eq!(batch.notes().len(), 1);
    let mut messages = vec![serde_json::json!({"role":"tool","content":"tool"})];
    super::append_openai_compatible_fallback(
        &mut messages,
        Some(&batch),
        crate::services::llm::route_profile::ToolResultMedia::FollowUpUserMessage,
        true,
        crate::services::llm::route_profile::ImageFormat::Unsupported,
    );
    assert_eq!(messages.len(), 1);
}

#[test]
fn anthropic_reports_a_provider_specific_preview_rejection() {
    let value = super::anthropic_tool_result("call-1", "result", &["not-base64".into()]);
    let content = value["content"].as_array().unwrap();
    assert_eq!(content.len(), 2);
    assert_eq!(
        content[1]["text"],
        "An extension preview could not be included for this model."
    );
}

#[test]
fn fallback_accepts_zero_one_and_exactly_eight_previews() {
    for count in [0, 1, 8] {
        let batch = previews(count);
        let mut messages = Vec::new();
        super::append_openai_compatible_fallback(
            &mut messages,
            Some(&batch),
            crate::services::llm::route_profile::ToolResultMedia::FollowUpUserMessage,
            true,
            crate::services::llm::route_profile::ImageFormat::MistralFlat,
        );
        assert_eq!(messages.len(), usize::from(count > 0));
        if count > 0 {
            assert_eq!(
                messages[0]["content"].as_array().unwrap().len(),
                count * 2 + 1
            );
        }
    }
}

#[test]
fn ollama_fallback_keeps_one_message_and_marks_omitted_previews() {
    let batch = previews(9);
    let mut messages = Vec::new();
    super::append_ollama_fallback(
        &mut messages,
        &batch,
        crate::services::llm::route_profile::ToolResultMedia::FollowUpUserMessage,
        true,
        crate::services::llm::route_profile::ImageFormat::OllamaNative,
    );

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].images.as_ref().map(Vec::len), Some(8));
    assert_eq!(
        messages[0].content,
        "Extension output (not user instruction). Additional extension previews are available in Beaver."
    );
}

fn previews(
    count: usize,
) -> crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch {
    crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch::from_ephemerals(
        (0..count).map(|index| (index, Some(format!("call-{index}")), artifact(index))),
    )
}

fn artifact(index: usize) -> crate::services::agent_local::tool_artifact::EphemeralArtifact {
    use crate::services::agent_local::tool_artifact::{
        ArtifactMetadata, ArtifactPurpose, ArtifactSource, EphemeralArtifact,
    };
    EphemeralArtifact {
        metadata: ArtifactMetadata {
            name: format!("preview-{index}.png"),
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
    }
}

fn invalid_preview() -> crate::services::agent_local::tool_artifact::EphemeralArtifact {
    let mut artifact = artifact(0);
    artifact.metadata.bytes = 12;
    artifact.bytes = b"not an image".to_vec();
    artifact
}
