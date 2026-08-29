pub(super) async fn ensure_reasoning_contract(
    provider: &str,
    model: &str,
    thinking_enabled: bool,
) -> Result<(), ()> {
    if !needs_refresh(provider, model, thinking_enabled) {
        return Ok(());
    }
    // Un réveil peut précéder le chargement du sélecteur après redémarrage.
    // La sonde catalogue restaure alors les capacités exactes sans générer de texte.
    crate::services::llm::model_catalog::list_models_for(provider)
        .await
        .map_err(|_| ())?;
    crate::services::llm::provider_model_lookup::resolve_local(provider, model)
        .filter(|capabilities| {
            capabilities.supports_thinking && !capabilities.reasoning_modes.is_empty()
        })
        .map(|_| ())
        .ok_or(())
}

fn needs_refresh(provider: &str, model: &str, thinking_enabled: bool) -> bool {
    thinking_enabled
        && crate::services::llm::route_profile::has_dynamic_reasoning_catalog(provider)
        && crate::services::llm::provider_model_lookup::resolve_local(provider, model)
            .is_none_or(|capabilities| capabilities.reasoning_modes.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_dynamic_anthropic_or_qwen_reasoning_needs_a_refresh() {
        assert!(needs_refresh("anthropic", "claude-not-loaded", true,));
        assert!(!needs_refresh("openai", "gpt-not-loaded", true,));
        assert!(!needs_refresh("qwen", "qwen-not-loaded", false,));
    }
}
