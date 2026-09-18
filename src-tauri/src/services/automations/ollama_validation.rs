use super::AutomationError;
use crate::services::agent_local::ollama_client::{OllamaClient, OllamaModelError};

// Bound the entire validation, including endpoint resolution, without changing generation budgets.
const VALIDATION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub(super) async fn validate_model(
    client: &OllamaClient,
    model: &str,
) -> Result<(), AutomationError> {
    if !crate::services::llm::runtime_models::valid_model_id(model) {
        return Err(AutomationError::ModelUnavailable);
    }
    // Ollama owns local availability and capabilities; the cloud catalogue cannot validate them.
    let info = tokio::time::timeout(VALIDATION_TIMEOUT, client.inspect_model(model))
        .await
        .map_err(|_| {
            ::log::warn!("[automations] ollama_validation=timeout");
            AutomationError::ProviderUnavailable
        })?
        .map_err(|error| match error {
            // An unreadable engine response does not establish that the model is absent.
            OllamaModelError::Unavailable | OllamaModelError::InvalidResponse => {
                AutomationError::ProviderUnavailable
            }
            OllamaModelError::NotFound => AutomationError::ModelUnavailable,
        })?;
    if !info
        .capabilities
        .iter()
        .any(|capability| capability == "tools")
    {
        return Err(AutomationError::ModelToolsUnsupported);
    }
    Ok(())
}
