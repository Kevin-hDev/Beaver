use crate::models::provider_contract::{ProviderCatalogEntry, ProviderCategory};
use crate::services::llm::{
    catalog::LLM_PROVIDERS, openai_compat::OpenAiCompatProvider, provider_model_lookup,
    tool_capable, types::ModelInfo,
};

#[tauri::command]
pub fn list_llm_providers_catalog() -> Vec<ProviderCatalogEntry> {
    LLM_PROVIDERS
        .iter()
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
    let all_free = is_provider_all_free(canonical_provider);
    for m in &mut models {
        let local_limits = provider_model_lookup::local_limits(canonical_provider, &m.id);
        if let Some(limits) = local_limits {
            m.context_length = limits.context_window;
            m.max_output_tokens = limits.max_output_tokens;
        }
        if let Some(caps) = provider_model_lookup::local_capabilities(canonical_provider, &m.id) {
            m.supports_tools = caps.supports_tools;
            m.supports_vision = caps.supports_vision;
            m.supports_thinking = caps.supports_thinking;
            m.reasoning_modes = crate::services::reasoning::supported_modes(
                canonical_provider,
                &m.id,
                m.supports_thinking,
            )
            .iter()
            .map(|mode| mode.to_string())
            .collect();
        } else {
            if let Some(caps) = provider_model_lookup::capabilities(canonical_provider, &m.id).await
            {
                m.supports_tools |= caps.supports_tools;
                m.supports_vision |= caps.supports_vision;
                m.supports_thinking |= caps.supports_thinking;
            }
            m.supports_tools |= tool_capable::supports_tools(canonical_provider, &m.id);
            m.supports_vision |= tool_capable::supports_vision(canonical_provider, &m.id);
            m.supports_thinking |= tool_capable::supports_thinking(canonical_provider, &m.id);
            if !m.supports_thinking {
                m.reasoning_modes.clear();
            } else if m.reasoning_modes.is_empty() {
                m.reasoning_modes =
                    crate::services::reasoning::supported_modes(canonical_provider, &m.id, true)
                        .iter()
                        .map(|mode| mode.to_string())
                        .collect();
            }
        }
        if m.default_reasoning_mode
            .as_ref()
            .is_some_and(|mode| !m.reasoning_modes.contains(mode))
        {
            m.default_reasoning_mode = None;
        }
        if all_free {
            m.is_free = true;
        } else if canonical_provider == "mistral" {
            m.is_free = is_mistral_free(&m.id);
        } else if canonical_provider == "zai" {
            m.is_free = is_zai_free(&m.id);
        }
    }
    crate::services::llm::runtime_models::replace_provider(canonical_provider, &models);
    Ok(models)
}

#[tauri::command]
pub async fn test_llm_connection(provider_id: String) -> Result<(), String> {
    let provider = OpenAiCompatProvider::new(&provider_id).map_err(String::from)?;
    provider.test_connection().await.map_err(String::from)
}

#[tauri::command]
pub async fn supports_tool_use(provider_id: String, model_id: String) -> bool {
    let canonical_provider = crate::services::llm::route::canonical_provider_id(&provider_id);
    if let Some(caps) = provider_model_lookup::local_capabilities(canonical_provider, &model_id) {
        return caps.supports_tools;
    }
    provider_model_lookup::capabilities(canonical_provider, &model_id)
        .await
        .is_some_and(|caps| caps.supports_tools)
        || crate::services::llm::runtime_models::lookup(canonical_provider, &model_id)
            .is_some_and(|model| model.supports_tools)
        || tool_capable::supports_tools(canonical_provider, &model_id)
}

#[tauri::command]
pub async fn get_provider_usage(
    connection_id: String,
    force_refresh: bool,
) -> Result<crate::services::provider_usage::ProviderUsageSnapshot, String> {
    crate::services::provider_usage::snapshot(&connection_id, force_refresh).await
}

fn is_provider_all_free(provider_id: &str) -> bool {
    matches!(provider_id, "groq" | "cerebras" | "google")
}

fn is_mistral_free(model_id: &str) -> bool {
    let id = model_id.to_lowercase();
    id.contains("devstral")
        || id.contains("magistral")
        || id.contains("ministral")
        || id.contains("pixtral")
        || id.contains("codestral-mamba")
        || id.contains("open-mistral")
        || id.contains("mistral-small")
}

fn is_zai_free(model_id: &str) -> bool {
    model_id.to_lowercase().contains("flash")
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn openrouter_model_keeps_upstream_tool_capability() {
        assert!(super::supports_tool_use("openrouter".to_string(), "openai/o3".to_string()).await);
    }
}
