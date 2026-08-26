use super::params::StreamTaskParams;
use crate::services::agent_local::types_ollama::OllamaThink;

pub(super) fn resolve(params: &StreamTaskParams) -> OllamaThink {
    canonical(
        &params.model,
        params.reasoning_mode.as_deref(),
        params.think,
        params.capability_hints.supports_thinking,
    )
}

pub(super) fn canonical(
    model: &str,
    reasoning_mode: Option<&str>,
    think: bool,
    _frontend_hint: Option<bool>,
) -> OllamaThink {
    let supports_thinking =
        crate::services::reasoning::provider_model_supports_thinking("ollama", model);
    let effective_mode = crate::services::reasoning::normalize_for_model(
        "ollama",
        model,
        reasoning_mode,
        supports_thinking,
    );
    crate::services::reasoning::ollama_think(
        model,
        effective_mode.as_deref(),
        think && supports_thinking,
    )
    .unwrap_or(OllamaThink::Bool(false))
}
