use super::*;
use crate::services::agent_local::tool_artifact::{
    ArtifactMetadata, ArtifactPurpose, ArtifactSource, EphemeralArtifact,
};
use crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch;
use crate::services::agent_local::types_ollama::{ChatMessage, ChatRequest, OllamaThink};

#[tokio::test]
async fn ollama_vision_payload_appends_one_verified_follow_up_after_tools() {
    let _guard = crate::services::llm::runtime_models::test_mutation_lock().await;
    install_model(true);
    let payload = final_payload();
    assert_eq!(payload["messages"], fixture()["vision"]);
    crate::services::llm::runtime_models::replace_provider("ollama", &[]);
}

#[tokio::test]
async fn ollama_text_payload_stays_text_only_when_vision_is_not_available() {
    let _guard = crate::services::llm::runtime_models::test_mutation_lock().await;
    install_model(false);
    let payload = final_payload();
    assert_eq!(payload["messages"], fixture()["text"]);
    crate::services::llm::runtime_models::replace_provider("ollama", &[]);
}

fn final_payload() -> serde_json::Value {
    let mut request = request();
    append_verified_previews(&mut request, &preview_batch());
    let wire = crate::services::agent_local::ollama_tool_role::wrap_tool_results(
        &request.messages,
        crate::services::llm::route_profile::payload_policy("ollama", &request.model)
            .expect("ollama policy")
            .message
            .tool_results,
    );
    crate::services::agent_local::ollama_wire::chat_request(&request, &wire).expect("payload")
}

fn request() -> ChatRequest {
    ChatRequest {
        model: "p6-vision".into(),
        messages: vec![ChatMessage::tool(
            "tool result".into(),
            Some("call-1".into()),
            None,
        )],
        stream: true,
        tools: None,
        options: None,
        keep_alive: None,
        think: Some(OllamaThink::Bool(false)),
        capture_reasoning: false,
        live_replay_target: None,
        fixture_candidate: None,
    }
}

fn preview_batch() -> ToolResultPreviewBatch {
    ToolResultPreviewBatch::from_ephemeral(
        0,
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
    )
}

fn install_model(supports_vision: bool) {
    crate::services::llm::runtime_models::replace_provider(
        "ollama",
        &[crate::services::llm::types::ModelInfo {
            id: "p6-vision".into(),
            display_name: None,
            owned_by: None,
            context_length: Some(8_192),
            max_output_tokens: Some(1_024),
            supports_tools: true,
            supports_vision,
            supports_thinking: false,
            supports_fast_mode: false,
            reasoning_modes: Vec::new(),
            default_reasoning_mode: None,
            context_usage_includes_reasoning: true,
            is_free: false,
        }],
    );
}

fn fixture() -> serde_json::Value {
    serde_json::from_slice(include_bytes!(
        "../../../test-fixtures/tool-result-media/ollama-fallback.json"
    ))
    .expect("fixture")
}
