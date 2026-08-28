use crate::models::provider_contract::{ProviderCatalogEntry, ProviderCategory};
use crate::services::llm::{
    catalog, openai_compat::OpenAiCompatProvider, provider_model_lookup, types::ModelInfo,
};

#[tauri::command]
pub fn list_llm_providers_catalog() -> Vec<ProviderCatalogEntry> {
    catalog::all()
        .into_iter()
        .map(|provider| {
            ProviderCatalogEntry::new(
                provider.id,
                provider.display_name,
                ProviderCategory::Llm,
                provider.signup_url,
                Some(provider.base_url),
                Some(provider.models_endpoint),
            )
        })
        .collect()
}

#[tauri::command]
pub async fn list_llm_models(provider_id: String) -> Result<Vec<ModelInfo>, String> {
    let provider = OpenAiCompatProvider::new(&provider_id).map_err(String::from)?;
    let canonical_provider = crate::services::llm::route::canonical_provider_id(&provider_id);
    let mut models = provider.list_models().await.map_err(String::from)?;
    models.truncate(500);
    let mut seen = std::collections::HashSet::new();
    models.retain(|m| seen.insert(m.id.clone()));
    let mut chat_filtered = Vec::with_capacity(models.len());
    for m in models {
        if provider_model_lookup::is_chat_model(canonical_provider, &m.id).await {
            chat_filtered.push(m);
        }
    }
    let mut models = chat_filtered;
    // Autorité dynamique brute : l'enrichissement ci-dessous sert uniquement à l'UI.
    let runtime_catalog = models.clone();
    for m in &mut models {
        m.context_usage_includes_reasoning =
            crate::services::llm::context_usage_includes_reasoning(&provider_id).unwrap_or(true);
        let remote_reasoning_modes = m.reasoning_modes.clone();
        let local = provider_model_lookup::local_capabilities(canonical_provider, &m.id).is_some();
        let resolved = provider_model_lookup::resolve(canonical_provider, &m.id).await;
        m.supports_fast_mode = resolved
            .as_ref()
            .is_some_and(|capabilities| capabilities.supports_fast_mode);
        let local_limits = provider_model_lookup::local_limits(canonical_provider, &m.id);
        if let Some(limits) = local_limits {
            m.context_length = limits.context_window;
            m.max_output_tokens = limits.max_output_tokens;
        }
        if let Some(capabilities) = resolved {
            if local {
                m.supports_tools = capabilities.supports_tools;
                m.supports_vision = capabilities.supports_vision;
                m.supports_thinking = capabilities.supports_thinking;
                m.reasoning_modes = crate::services::reasoning::restrict_to_dynamic_modes(
                    capabilities.reasoning_modes.clone(),
                    (!remote_reasoning_modes.is_empty())
                        .then_some(remote_reasoning_modes.as_slice()),
                );
            } else {
                m.supports_tools |= capabilities.supports_tools;
                m.supports_vision |= capabilities.supports_vision;
                m.supports_thinking |= capabilities.supports_thinking;
            }
            if !m.supports_thinking {
                m.reasoning_modes.clear();
            } else if m.reasoning_modes.is_empty() {
                m.reasoning_modes = capabilities.reasoning_modes;
            }
            m.default_reasoning_mode = m
                .default_reasoning_mode
                .take()
                .filter(|mode| m.reasoning_modes.contains(mode))
                .or_else(|| {
                    capabilities
                        .default_reasoning_mode
                        .filter(|mode| m.reasoning_modes.contains(mode))
                });
        }
        if m.default_reasoning_mode
            .as_ref()
            .is_some_and(|mode| !m.reasoning_modes.contains(mode))
        {
            m.default_reasoning_mode = None;
        }
    }
    crate::services::llm::runtime_models::replace_provider(canonical_provider, &runtime_catalog);
    Ok(models)
}

#[tauri::command]
pub async fn get_model_context(
    route_id: String,
    model_id: String,
    ollama: tauri::State<'_, crate::services::agent_local::ollama_client::OllamaClient>,
) -> Result<Option<u64>, String> {
    use crate::services::reasoning_continuity::contract::RouteId;

    crate::services::reasoning_continuity::limits::validate_model_id(&model_id)
        .map_err(|_| "Modèle invalide".to_string())?;
    let route =
        RouteId::from_provider_id(&route_id).ok_or_else(|| "Fournisseur invalide".to_string())?;
    if route == RouteId::Ollama {
        return resolve_ollama_context(&ollama, &model_id).await.map(Some);
    }
    Ok(crate::services::llm::model_context_length(&route_id, &model_id).await)
}

async fn resolve_ollama_context(
    ollama: &crate::services::agent_local::ollama_client::OllamaClient,
    model_id: &str,
) -> Result<u64, String> {
    if let Some(loaded) = ollama.loaded_context_length(model_id).await? {
        if loaded > 0 {
            return Ok(loaded);
        }
    }
    let info = ollama.show_model(model_id).await?;
    let parsed = crate::services::agent_local::modelfile_parser::parse_modelfile(&info.modelfile);
    if let Some(configured) = parsed
        .parameters
        .get("num_ctx")
        .and_then(|value| value.as_u64())
    {
        if configured > 0 {
            return Ok(configured);
        }
    }
    let effective = u64::from(crate::services::gpu_detect::compute_default_num_ctx());
    Ok(match (info.context_length, effective) {
        (model, configured) if model > 0 && configured > 0 => model.min(configured),
        (model, _) if model > 0 => model,
        (_, configured) => configured,
    })
}

#[tauri::command]
pub async fn test_llm_connection(provider_id: String) -> Result<(), String> {
    let provider = OpenAiCompatProvider::new(&provider_id).map_err(String::from)?;
    provider.test_connection().await.map_err(String::from)
}

#[tauri::command]
pub async fn supports_tool_use(provider_id: String, model_id: String) -> bool {
    let canonical_provider = crate::services::llm::route::canonical_provider_id(&provider_id);
    provider_model_lookup::resolve(canonical_provider, &model_id)
        .await
        .is_some_and(|capabilities| capabilities.supports_tools)
}

#[tauri::command]
pub async fn get_provider_usage(
    connection_id: String,
    force_refresh: bool,
) -> Result<crate::services::provider_usage::ProviderUsageSnapshot, String> {
    crate::services::provider_usage::snapshot(&connection_id, force_refresh).await
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn openrouter_model_keeps_upstream_tool_capability() {
        assert!(super::supports_tool_use("openrouter".to_string(), "openai/o3".to_string()).await);
    }

    #[tokio::test]
    async fn model_context_comes_from_the_registered_route_metadata() {
        assert_eq!(
            crate::services::llm::model_context_length("openai", "gpt-5.6-sol").await,
            Some(1_050_000)
        );
        assert_eq!(
            crate::services::llm::model_context_length("openai", "unknown-model").await,
            None
        );
    }
}
