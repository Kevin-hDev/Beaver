use super::tool_artifact_preview::ToolResultPreviewBatch;
use super::types_ollama::ChatRequest;

/// P6 projects verified bytes only for Ollama models whose active capability
/// record declares vision; unknown models stay text-only.
pub(super) fn append_verified_previews(
    request: &mut ChatRequest,
    previews: &ToolResultPreviewBatch,
) {
    let Some(policy) =
        crate::services::llm::route_profile::payload_policy("ollama", &request.model)
    else {
        return;
    };
    let supports_vision =
        crate::services::llm::provider_model_lookup::resolve_local("ollama", &request.model)
            .is_some_and(|capabilities| capabilities.supports_vision);
    crate::services::llm::tool_result_projection::append_ollama_fallback(
        &mut request.messages,
        previews,
        policy.tool_result_media,
        supports_vision,
        policy.message.images,
    );
}

#[cfg(test)]
#[path = "agent_loop_ollama_media_tests.rs"]
mod tests;
