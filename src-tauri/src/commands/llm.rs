use crate::services::llm::{
    catalog::{ProviderSpec, LLM_PROVIDERS},
    model_registry_lookup,
    openai_compat::OpenAiCompatProvider,
    tool_capable,
    types::ModelInfo,
};

#[tauri::command]
pub fn list_llm_providers_catalog() -> Vec<ProviderSpec> {
    LLM_PROVIDERS.to_vec()
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
        if model_registry_lookup::is_chat_model(canonical_provider, &m.id).await {
            chat_filtered.push(m);
        }
    }
    let mut models = chat_filtered;
    let all_free = is_provider_all_free(canonical_provider);
    for m in &mut models {
        match model_registry_lookup::capabilities(canonical_provider, &m.id).await {
            Some(caps) => {
                m.supports_tools = m.supports_tools || caps.supports_tools;
                m.supports_vision = m.supports_vision
                    || caps.supports_vision
                    || tool_capable::supports_vision(canonical_provider, &m.id);
                m.supports_thinking = m.supports_thinking
                    || caps.supports_thinking
                    || tool_capable::supports_thinking(canonical_provider, &m.id);
            }
            None => {
                if !m.supports_tools {
                    m.supports_tools = tool_capable::supports_tools(canonical_provider, &m.id);
                }
                if !m.supports_vision {
                    m.supports_vision = tool_capable::supports_vision(canonical_provider, &m.id);
                }
                if !m.supports_thinking {
                    m.supports_thinking =
                        tool_capable::supports_thinking(canonical_provider, &m.id);
                }
            }
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
    model_registry_lookup::capabilities(canonical_provider, &model_id)
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
