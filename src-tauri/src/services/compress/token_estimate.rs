use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_session::AgentMessage;

pub use super::token_estimate_request::{
    estimate_request_tokens, estimate_request_tokens_for_provider,
    estimate_textual_request_tokens_for_provider, estimate_tool_tokens,
};

pub fn estimate_tokens(messages: &[ChatMessage]) -> usize {
    crate::services::token_counting::estimate_chat_tokens(messages)
}

pub fn estimate_tokens_for_provider(provider_id: &str, messages: &[ChatMessage]) -> usize {
    let visible = if counts_visible_reasoning(provider_id) {
        estimate_tokens(messages)
    } else {
        crate::services::token_counting::estimate_chat_tokens_without_reasoning(messages)
    };
    visible.saturating_add(
        super::token_estimate_reasoning::estimate_native_continuation_tokens(provider_id, messages),
    )
}

pub fn estimate_textual_tokens_for_provider(provider_id: &str, messages: &[ChatMessage]) -> usize {
    let visible = if counts_visible_reasoning(provider_id) {
        crate::services::token_counting::estimate_textual_chat_tokens(messages)
    } else {
        messages
            .iter()
            .map(crate::services::token_counting::estimate_textual_chat_message_tokens_without_reasoning)
            .sum()
    };
    visible.saturating_add(
        super::token_estimate_reasoning::estimate_native_continuation_tokens(provider_id, messages),
    )
}

pub fn estimate_message_tokens_for_provider(provider_id: &str, message: &ChatMessage) -> usize {
    let visible = if counts_visible_reasoning(provider_id) {
        crate::services::token_counting::estimate_chat_message_tokens(message)
    } else {
        crate::services::token_counting::estimate_chat_message_tokens_without_reasoning(message)
    };
    visible.saturating_add(
        super::token_estimate_reasoning::estimate_native_continuation_tokens(
            provider_id,
            std::slice::from_ref(message),
        ),
    )
}

fn counts_visible_reasoning(provider_id: &str) -> bool {
    provider_id != crate::services::codex_client::PROVIDER_ID
}

pub(crate) fn estimate_native_continuation_tokens_for_provider(
    provider_id: &str,
    messages: &[ChatMessage],
) -> usize {
    super::token_estimate_reasoning::estimate_native_continuation_tokens(provider_id, messages)
}

pub fn estimate_checkpoint_message_tokens(message: &AgentMessage) -> u32 {
    let visible = crate::services::token_counting::estimate_agent_message_tokens(message);
    let continuation = message
        .continuation
        .as_ref()
        .and_then(|envelope| serde_json::to_vec(envelope).ok())
        .map_or(0, |bytes| bytes.len().div_ceil(4));
    visible.saturating_add(continuation).min(u32::MAX as usize) as u32
}

pub fn should_compress(used_tokens: usize, context_window: u64, threshold_pct: u8) -> bool {
    if threshold_pct == 0 {
        return false;
    }
    let limit = (context_window as f64 * threshold_pct as f64 / 100.0) as usize;
    used_tokens >= limit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent_local::types_ollama::ChatMessage;

    fn msg(role: &str, content: &str) -> ChatMessage {
        match role {
            "system" => ChatMessage::system(content.to_string()),
            "user" => ChatMessage::user(content.to_string()),
            "assistant" => ChatMessage::assistant(content.to_string(), None, None, None, None),
            "tool" => ChatMessage::tool(content.to_string(), None, None),
            other => panic!("unsupported chat role in test/setup: {other}"),
        }
    }

    #[test]
    fn estimate_empty() {
        assert_eq!(estimate_tokens(&[]), 0);
    }

    #[test]
    fn estimate_simple_message() {
        let msgs = vec![msg("user", &"a".repeat(400))];
        assert_eq!(estimate_tokens(&msgs), 100);
    }

    #[test]
    fn estimate_multiple_messages() {
        let msgs = vec![
            msg("user", &"a".repeat(400)),
            msg("assistant", &"b".repeat(800)),
            msg("tool", &"c".repeat(1200)),
        ];
        assert_eq!(estimate_tokens(&msgs), 600);
    }

    #[test]
    fn threshold_check() {
        assert!(!should_compress(80_000, 100_000, 85));
        assert!(should_compress(86_000, 100_000, 85));
    }

    #[test]
    fn threshold_zero_means_never() {
        assert!(!should_compress(99_000, 100_000, 0));
    }

    #[test]
    fn threshold_100_means_at_max() {
        assert!(!should_compress(99_999, 100_000, 100));
        assert!(should_compress(100_000, 100_000, 100));
    }

    #[test]
    fn estimate_counts_images() {
        let mut message = msg("user", "hello");
        message.images = Some(vec!["iVBORw0KGgo=".to_string()]);
        assert!(estimate_tokens(&[message]) >= crate::services::llm::vision::IMAGE_TOKEN_ESTIMATE);
    }

    #[test]
    fn textual_request_tracks_images_separately() {
        let mut message = msg("user", "hello");
        message.images = Some(vec!["iVBORw0KGgo=".to_string()]);

        let capacity = estimate_request_tokens_for_provider("ollama", &[message.clone()], &[]);
        let textual = estimate_textual_request_tokens_for_provider("ollama", &[message], &[]);

        assert_eq!(
            capacity - textual,
            crate::services::llm::vision::IMAGE_TOKEN_ESTIMATE
        );
    }

    #[test]
    fn estimate_counts_cjk_conservatively() {
        let msgs = vec![msg("user", &"你".repeat(1000))];
        assert_eq!(estimate_tokens(&msgs), 1250);
    }

    #[test]
    fn estimate_request_includes_tool_definitions() {
        let messages = vec![msg("user", "hello")];
        let tools = vec![serde_json::json!({
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "Read a file",
                "parameters": {"type": "object", "properties": {}}
            }
        })];

        assert!(estimate_request_tokens(&messages, &tools) > estimate_tokens(&messages));
    }

    #[test]
    fn codex_request_excludes_reasoning_that_is_not_replayed() {
        let mut message = msg("assistant", "answer");
        message.tool_loop_reasoning = Some("hidden reasoning".repeat(100));
        let messages = [message];

        assert!(
            estimate_request_tokens_for_provider(
                crate::services::codex_client::PROVIDER_ID,
                &messages,
                &[],
            ) < estimate_request_tokens(&messages, &[])
        );
    }

    #[test]
    fn request_budget_counts_retained_native_continuation() {
        use crate::services::reasoning_continuity::contract::{
            ContractId, CredentialScope, ReasoningModeId, RouteId,
        };
        use crate::services::reasoning_continuity::envelope::{
            CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
        };

        let mut message = msg("assistant", "answer");
        message.continuation = Some(ReasoningEnvelope::new(
            ContractId::OllamaNativeV1,
            ReasoningSource {
                route_id: RouteId::Ollama,
                model_id: "fixture".into(),
                credential_scope: CredentialScope::local_uncredentialed(),
                reasoning_mode: ReasoningModeId::Auto,
            },
            CompletionState::Complete,
            ContinuationState::OllamaNative {
                thinking: "native state ".repeat(200),
            },
            Vec::new(),
        ));

        assert!(
            estimate_tokens_for_provider("ollama", &[message.clone()])
                > estimate_tokens(&[message])
        );
    }
}
