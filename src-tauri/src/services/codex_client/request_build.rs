use super::types::CodexRequest;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::fast_mode::FastModeRequest;

pub(super) fn build_codex_request(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    session_id: Option<&str>,
    fast_mode: FastModeRequest,
) -> CodexRequest {
    super::request::build_codex_request_with_continuity(
        model,
        messages,
        tools,
        reasoning_mode,
        session_id,
        fast_mode,
        None,
    )
    .expect("a request without a continuation target cannot be rejected")
}
