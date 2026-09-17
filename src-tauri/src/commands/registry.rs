use crate::services::llm::litellm_catalog_search::{self, FamilyGroup, RegistryModelInfo};

#[tauri::command]
pub async fn search_registry(query: String) -> Vec<RegistryModelInfo> {
    litellm_catalog_search::search(&query, 100).await
}

#[tauri::command]
pub async fn list_registry_families() -> Vec<FamilyGroup> {
    litellm_catalog_search::list_families().await
}

#[tauri::command]
pub async fn list_family_models(family: String) -> Vec<RegistryModelInfo> {
    litellm_catalog_search::list_family_models(&family).await
}
