use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::compress::prompt;
use crate::services::compress::token_estimate;

pub const BOUNDARY_CONTENT: &str =
    "[Compression boundary — previous messages have been summarized]";

/// Décide si l'auto-compression doit se déclencher.
pub fn should_auto_compress(
    enabled: bool,
    _native_context: u64,
    configured_context: u64,
    used_tokens: usize,
    threshold_pct: u8,
) -> bool {
    if !enabled {
        return false;
    }
    if configured_context == 0 {
        return false;
    }
    token_estimate::should_compress(used_tokens, configured_context, threshold_pct)
}

/// Filtre les messages pour la requête de compression (exclut system).
pub fn prepare_messages_for_compression(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    messages
        .iter()
        .filter(|m| m.role != "system")
        .cloned()
        .collect()
}

/// Construit les messages à envoyer au LLM pour obtenir le résumé.
/// Ajoute le prompt de compression comme dernier message user.
pub fn build_compression_request_content(
    messages: &[ChatMessage],
    custom_instructions: Option<&str>,
) -> Vec<ChatMessage> {
    let mut prepared = prepare_messages_for_compression(messages);
    prepared.push(ChatMessage {
        role: "user".to_string(),
        content: prompt::build_compression_prompt(custom_instructions),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        reasoning_content: None,
    });
    prepared
}

/// Construit les messages post-compression : boundary marker + résumé.
pub fn build_post_compression_messages(
    summary: &str,
    suppress_follow_up: bool,
) -> Vec<ChatMessage> {
    let boundary = ChatMessage {
        role: "system".to_string(),
        content: BOUNDARY_CONTENT.to_string(),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        reasoning_content: None,
    };
    let summary_msg = ChatMessage {
        role: "user".to_string(),
        content: prompt::format_summary_message(summary, suppress_follow_up),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        reasoning_content: None,
    };
    vec![boundary, summary_msg]
}

/// Applique la compression : remplace les messages par le system prompt original + boundary + résumé.
/// Retourne le nombre de messages avant compression.
#[cfg(test)]
pub fn apply_compression(
    messages: &mut Vec<ChatMessage>,
    summary: &str,
    suppress_follow_up: bool,
) -> usize {
    let system_msg = messages.iter().find(|m| m.role == "system").cloned();
    let pre_count = messages.len();
    let post_messages = build_post_compression_messages(summary, suppress_follow_up);
    messages.clear();
    if let Some(sys) = system_msg {
        messages.push(sys);
    }
    messages.extend(post_messages);
    pre_count
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
