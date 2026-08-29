use crate::models::provider_contract::{ProviderCatalogEntry, ProviderCategory};
use crate::services::llm::{catalog, provider_model_lookup, types::ModelInfo};

#[tauri::command]
pub fn list_llm_providers_catalog() -> Vec<ProviderCatalogEntry> {
    provider_catalog_entries(catalog::all())
}

#[tauri::command]
pub fn list_llm_configurable_providers_catalog() -> Vec<ProviderCatalogEntry> {
    provider_catalog_entries(catalog::configurable())
}

fn provider_catalog_entries(providers: Vec<catalog::ProviderSpec>) -> Vec<ProviderCatalogEntry> {
    providers
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
    crate::services::llm::model_catalog::list_models_for(&provider_id)
        .await
        .map_err(String::from)
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
    let loaded = ollama.loaded_context_length(model_id).await?;
    if loaded.is_some_and(|value| value > 0) {
        return Ok(select_ollama_context(loaded, None, 0, 0));
    }
    let info = ollama.show_model(model_id).await?;
    let parsed = crate::services::agent_local::modelfile_parser::parse_modelfile(&info.modelfile);
    let configured = parsed
        .parameters
        .get("num_ctx")
        .and_then(|value| value.as_u64());
    if configured.is_some_and(|value| value > 0) {
        return Ok(select_ollama_context(loaded, configured, 0, 0));
    }
    let effective = u64::from(crate::services::gpu_detect::compute_default_num_ctx());
    Ok(select_ollama_context(
        loaded,
        configured,
        info.context_length,
        effective,
    ))
}

fn select_ollama_context(
    loaded: Option<u64>,
    configured: Option<u64>,
    model: u64,
    effective: u64,
) -> u64 {
    if let Some(value) = loaded.filter(|value| *value > 0) {
        return value;
    }
    if let Some(value) = configured.filter(|value| *value > 0) {
        return value;
    }
    match (model, effective) {
        (model, configured) if model > 0 && configured > 0 => model.min(configured),
        (model, _) if model > 0 => model,
        (_, configured) => configured,
    }
}

#[tauri::command]
pub async fn test_llm_connection(provider_id: String) -> Result<(), String> {
    crate::services::llm::model_catalog::test_connection_for(&provider_id)
        .await
        .map_err(String::from)
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
#[path = "llm_tests.rs"]
mod tests;
