//! Commandes Tauri pour le module Search / Scraping multi-provider.

use crate::services::search::{
    catalog::{SearchProviderSpec, SEARCH_PROVIDERS},
    test_connection,
};

#[tauri::command]
pub fn list_search_providers_catalog() -> Vec<SearchProviderSpec> {
    SEARCH_PROVIDERS.to_vec()
}

#[tauri::command]
pub async fn test_search_connection(provider_id: String) -> Result<(), String> {
    test_connection(&provider_id).await
}
