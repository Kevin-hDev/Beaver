use super::*;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};
use serde_json::json;

#[test]
fn result_counts_survive_secret_redaction_without_exposing_model_text() {
    let result = StreamResult {
        content: "sk-private-answer".into(),
        thinking: "private-thought".into(),
        usage: crate::services::provider_usage::RequestUsage::from_json_with_context(
            &json!({"input_tokens":2048,"output_tokens":5,
                "input_tokens_details":{"cached_tokens":1024}}),
            crate::services::provider_usage::UsageContext::responses("codex-oauth", "gpt-6-astra"),
        ),
        ..Default::default()
    };
    let summary = support::clip(&result_summary(0, &result));
    assert!(summary.contains("cache_read_count=1024"), "{summary}");
    assert!(summary.contains("cache_write_count=unknown"), "{summary}");
    assert!(!summary.contains("sk-private-answer"));
    assert!(!summary.contains("private-thought"));
}

#[test]
fn request_stats_counts_reasoning_without_content() {
    let messages = vec![
        ChatMessage::assistant(
            "".to_string(),
            Some("réflexion".to_string()),
            None,
            Some("réflexion".to_string()),
            Some(vec![ToolCallOllama {
                id: Some("call_1".to_string()),
                extra_content: None,
                function: ToolCallFunction {
                    name: "grep".to_string(),
                    arguments: json!({"pattern": "x"}),
                },
            }]),
        ),
        ChatMessage::tool("ok".to_string(), Some("call_1".to_string()), None),
    ];

    assert_eq!(
        request_stats(&messages),
        ModelRequestStats {
            messages: 2,
            assistant_messages: 1,
            assistant_reasoning_messages: 1,
            assistant_reasoning_chars: 9,
            assistant_content_chars: 0,
            assistant_tool_calls: 1,
            tool_messages: 1,
        }
    );
}

#[test]
fn char_count_is_utf8_safe() {
    assert_eq!(char_count("é🙂x"), 3);
}

#[test]
fn request_stats_counts_persisted_ollama_continuity() {
    use crate::services::reasoning_continuity::contract::{
        ContractId, CredentialScope, ReasoningModeId, RouteId,
    };
    use crate::services::reasoning_continuity::envelope::{
        CompletionState, ReasoningEnvelope, ReasoningSource,
    };

    let continuation = ReasoningEnvelope::new(
        ContractId::OllamaNativeV1,
        ReasoningSource {
            route_id: RouteId::Ollama,
            model_id: "qwen3.5:4b".into(),
            credential_scope: CredentialScope::local_uncredentialed(),
            reasoning_mode: ReasoningModeId::Auto,
        },
        CompletionState::Complete,
        ContinuationState::OllamaNative {
            thinking: "raisonnement durable".into(),
        },
        Vec::new(),
    );
    let messages = [ChatMessage::assistant(
        "réponse".into(),
        None,
        Some(continuation),
        None,
        None,
    )];

    let stats = request_stats(&messages);
    assert_eq!(stats.assistant_reasoning_messages, 1);
    assert_eq!(stats.assistant_reasoning_chars, 20);
}

#[test]
fn request_stats_never_turns_anthropic_opaque_blocks_into_diagnostic_text() {
    use crate::services::reasoning_continuity::contract::{
        ContractId, CredentialScope, ReasoningModeId, RouteId,
    };
    use crate::services::reasoning_continuity::envelope::{
        CompletionState, ReasoningEnvelope, ReasoningSource,
    };

    let continuation = ReasoningEnvelope::new(
        ContractId::AnthropicMessagesV1,
        ReasoningSource {
            route_id: RouteId::Anthropic,
            model_id: "claude-haiku-4-5-20251001".into(),
            credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
            reasoning_mode: ReasoningModeId::Low,
        },
        CompletionState::Complete,
        ContinuationState::AnthropicBlocks {
            blocks: vec![serde_json::json!({
                "type":"thinking",
                "thinking":"opaque-secret",
                "signature":"AAE+/=="
            })],
        },
        Vec::new(),
    );
    let messages = [ChatMessage::assistant(
        "answer".into(),
        None,
        Some(continuation),
        None,
        None,
    )];

    let stats = request_stats(&messages);
    assert_eq!(stats.assistant_reasoning_messages, 0);
    assert_eq!(stats.assistant_reasoning_chars, 0);
    assert_eq!(stats.assistant_content_chars, 6);
}
