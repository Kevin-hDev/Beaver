use super::*;
use crate::services::agent_local::conversation_attachments::ResolvedImage;
use crate::services::agent_local::types_message::{ToolCallRequest, ToolCallRequestFunction};
use crate::services::reasoning_continuity::contract::{
    ContractId, CredentialScope, ReasoningModeId, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};

#[test]
fn canonical_adapter_preserves_native_fields_without_rebuilding_reasoning() {
    let continuation = ReasoningEnvelope::new(
        ContractId::OllamaNativeV1,
        ReasoningSource {
            route_id: RouteId::Ollama,
            model_id: "qwen3.5:4b".into(),
            credential_scope: CredentialScope::local_uncredentialed(),
            reasoning_mode: ReasoningModeId::High,
        },
        CompletionState::Complete,
        ContinuationState::OllamaNative {
            thinking: "opaque-native".into(),
        },
        Vec::new(),
    );
    let converted = convert(message(Some(continuation.clone()), None)).unwrap();

    assert_eq!(
        converted.images.as_deref(),
        Some(&["image-base64".to_string()][..])
    );
    let call = &converted.tool_calls.as_ref().unwrap()[0];
    assert_eq!(call.id.as_deref(), Some("call-provider-1"));
    assert_eq!(
        call.extra_content,
        Some(serde_json::json!({"signature": "opaque"}))
    );
    assert_eq!(call.function.name, "bash");
    assert_eq!(converted.tool_call_id.as_deref(), Some("call-parent"));
    assert_eq!(converted.display_thinking.as_deref(), Some("visible only"));
    assert_eq!(converted.continuation, Some(continuation));
    assert!(converted.tool_loop_reasoning.is_none());
}

#[test]
fn canonical_adapter_fails_closed_on_legacy_reasoning() {
    assert!(convert(message(None, Some("forged legacy".into()))).is_err());
}

fn message(continuation: Option<ReasoningEnvelope>, legacy: Option<String>) -> ProviderMessage {
    ProviderMessage {
        message_id: Some("message-1".into()),
        turn_id: "turn-1".into(),
        role: ProviderRole::Assistant,
        content: "visible answer".into(),
        images: vec![ResolvedImage {
            mime_type: "image/png".into(),
            base64: "image-base64".into(),
        }],
        tool_calls: Some(vec![ToolCallRequest {
            id: "call-provider-1".into(),
            extra_content: Some(serde_json::json!({"signature": "opaque"})),
            function: ToolCallRequestFunction {
                name: "bash".into(),
                arguments: serde_json::json!({"cmd": "pwd"}),
            },
        }]),
        tool_name: Some("bash".into()),
        tool_call_id: Some("call-parent".into()),
        display_thinking: Some("visible only".into()),
        continuation,
        tool_loop_reasoning: legacy,
        continuity_barrier_before: false,
    }
}
