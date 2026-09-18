use super::AutomationError;
use crate::services::agent_local::ollama_client::OllamaClient;

pub(super) async fn validate_model(
    client: &OllamaClient,
    model: &str,
) -> Result<(), AutomationError> {
    if !crate::services::llm::runtime_models::valid_model_id(model) {
        return Err(AutomationError::ModelUnavailable);
    }
    // Ollama owns local availability and capabilities; the cloud catalogue cannot validate them.
    let info = client
        .show_model(model)
        .await
        .map_err(|_| AutomationError::ModelUnavailable)?;
    if !info
        .capabilities
        .iter()
        .any(|capability| capability == "tools")
    {
        return Err(AutomationError::ModelToolsUnsupported);
    }
    Ok(())
}
