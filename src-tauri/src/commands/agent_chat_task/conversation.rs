use crate::services::agent_local::conversation_admission::AdmittedTurn;
use crate::services::agent_local::conversation_history::{ProviderMessage, ProviderRole};
use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};

pub(crate) enum StreamConversation {
    Canonical(AdmittedTurn),
    #[deprecated(note = "Tasks 11-13 migrate remaining internal producers")]
    InternalLegacy(Vec<ChatMessage>),
}

impl StreamConversation {
    pub(crate) fn canonical(admitted: AdmittedTurn) -> Self {
        Self::Canonical(admitted)
    }

    #[allow(deprecated, reason = "Tasks 11-13 migrate non-IPC internal producers")]
    pub(crate) fn internal_legacy(messages: Vec<ChatMessage>) -> Self {
        Self::InternalLegacy(messages)
    }

    pub(crate) fn into_messages(self) -> Result<Vec<ChatMessage>, String> {
        match self {
            Self::Canonical(admitted) => {
                admitted.history.messages.into_iter().map(convert).collect()
            }
            #[allow(deprecated)]
            Self::InternalLegacy(messages) => Ok(messages),
        }
    }
}

fn convert(message: ProviderMessage) -> Result<ChatMessage, String> {
    let images = (!message.images.is_empty()).then(|| {
        message
            .images
            .into_iter()
            .map(|image| image.base64)
            .collect()
    });
    let role = match message.role {
        ProviderRole::User => "user",
        ProviderRole::Assistant => "assistant",
        ProviderRole::Tool => "tool",
    };
    let tool_calls = message.tool_calls.map(|calls| {
        calls
            .into_iter()
            .map(|call| ToolCallOllama {
                id: Some(call.id),
                extra_content: call.extra_content,
                function: ToolCallFunction {
                    name: call.function.name,
                    arguments: call.function.arguments,
                },
            })
            .collect()
    });
    if message.legacy_tool_loop_reasoning.is_some() {
        return Err(generic_error());
    }
    Ok(ChatMessage {
        role: role.to_string(),
        content: message.content,
        images,
        tool_calls,
        tool_name: message.tool_name,
        tool_call_id: message.tool_call_id,
        display_thinking: message.display_thinking,
        continuation: message.continuation,
        legacy_tool_loop_reasoning: None,
    })
}

fn generic_error() -> String {
    "conversation_admission_failed".to_string()
}

#[cfg(test)]
mod tests {
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
        assert!(converted.legacy_tool_loop_reasoning.is_none());
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
            files: Vec::new(),
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
            legacy_tool_loop_reasoning: legacy,
            skill_id: None,
            skill_name: None,
            continuity_barrier_before: false,
        }
    }
}
