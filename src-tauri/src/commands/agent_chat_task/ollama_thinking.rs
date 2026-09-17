use super::params::StreamTaskParams;
use crate::services::agent_local::types_ollama::OllamaThink;

pub(super) async fn resolve(params: &StreamTaskParams) -> Result<OllamaThink, String> {
    params
        .reasoning_profile
        .ollama_payload
        .clone()
        .ok_or_else(|| "conversation_admission_failed".to_string())
}

#[cfg(test)]
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
