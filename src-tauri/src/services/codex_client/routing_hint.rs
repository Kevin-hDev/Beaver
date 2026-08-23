use super::types::CodexRequest;

const MAX_ROUTING_HINT_BYTES: usize = 160;
const INVALID_CONFIGURATION: &str = "provider_configuration_invalid";

pub(super) fn for_request(request: &CodexRequest) -> Result<String, String> {
    if !crate::services::llm::runtime_models::valid_model_id(&request.model) {
        return Err(INVALID_CONFIGURATION.to_string());
    }
    let suffix = match request.service_tier.as_deref() {
        None => "",
        Some("priority") => ";tier=priority",
        Some(_) => return Err(INVALID_CONFIGURATION.to_string()),
    };
    // Le client Codex officiel envoie cet en-tête sur HTTP et WebSocket.
    // Le dériver du payload canonique empêche le corps et le routage de diverger.
    let hint = format!("model={}{}", request.model, suffix);
    if hint.len() > MAX_ROUTING_HINT_BYTES {
        return Err(INVALID_CONFIGURATION.to_string());
    }
    Ok(hint)
}
