use crate::models::provider_contract::{ProviderCatalogEntry, ProviderCategory};
use crate::services::llm::{catalog, provider_model_lookup, types::ModelInfo};

#[tauri::command]
/// Catalogue public utilisé par les sélecteurs de modèle et les écrans de chat.
pub fn list_llm_providers_catalog() -> Vec<ProviderCatalogEntry> {
    provider_catalog_entries(catalog::all())
}

#[tauri::command]
/// Catalogue élargi utilisé par Réglages pour configurer les routes candidates.
pub fn list_llm_configurable_providers_catalog() -> Vec<ProviderCatalogEntry> {
    provider_catalog_entries(catalog::configurable())
}

fn provider_catalog_entries(providers: Vec<catalog::ProviderSpec>) -> Vec<ProviderCatalogEntry> {
    providers
        .into_iter()
        .map(|provider| {
            let mut entry = ProviderCatalogEntry::new(
                provider.id,
                provider.display_name,
                ProviderCategory::Llm,
                provider.signup_url,
                provider.base_url,
                provider.models_endpoint,
            );
            entry.connection_kind = provider.connection_kind;
            entry
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
        let context = crate::services::compress::context_resolve::resolve_ollama_with_client(
            &ollama, &model_id,
        )
        .await;
        return Ok((context.configured > 0).then_some(context.configured));
    }
    Ok(crate::services::llm::model_context_length(&route_id, &model_id).await)
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
