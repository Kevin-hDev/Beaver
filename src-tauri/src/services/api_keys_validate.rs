const MAX_KEY_LEN: usize = 256;

fn is_known_provider(id: &str) -> bool {
    crate::services::llm::catalog::find(id).is_some()
        || crate::services::search::catalog::SEARCH_PROVIDERS
            .iter()
            .any(|p| p.id == id)
        || crate::services::forecast::catalog::FORECAST_PROVIDERS
            .iter()
            .any(|p| p.id == id && p.requires_api_key)
}

pub fn validate_provider(provider_id: &str) -> Result<(), String> {
    if !is_known_provider(provider_id) {
        return Err("provider inconnu".to_string());
    }
    Ok(())
}

pub fn validate_key_input(provider_id: &str, key: &str) -> Result<(), String> {
    validate_provider(provider_id)?;
    if key.is_empty() || key.len() > MAX_KEY_LEN {
        return Err("clé API invalide (vide ou trop longue)".to_string());
    }
    if key.bytes().any(|b| b < 0x20 && b != b'\n') {
        return Err("clé API contient des caractères de contrôle".to_string());
    }
    Ok(())
}
