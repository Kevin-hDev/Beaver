use super::params::StreamTaskParams;
use crate::services::agent_local::types_ollama::OllamaThink;

pub(super) async fn resolve(params: &StreamTaskParams) -> Result<OllamaThink, String> {
    if params.continuation_target.is_some() {
        return params
            .ollama_reasoning
            .as_ref()
            .map(|effective| effective.payload.clone())
            .ok_or_else(|| "conversation_admission_failed".to_string());
    }
    let info = crate::services::agent_local::ollama_client::OllamaClient::from_global()?
        .show_model(&params.model)
        .await
        .map_err(|_| "conversation_admission_failed".to_string())?;
    canonical(
        &params.model,
        params.reasoning_mode.as_deref(),
        params.think,
        Some(&info.capabilities),
    )
}

pub(super) fn canonical(
    model: &str,
    reasoning_mode: Option<&str>,
    think: bool,
    capabilities: Option<&[String]>,
) -> Result<OllamaThink, String> {
    crate::services::reasoning_ollama::resolve(model, reasoning_mode, think, capabilities)
        .map(|effective| effective.payload)
        .map_err(|_| "conversation_admission_failed".to_string())
}
