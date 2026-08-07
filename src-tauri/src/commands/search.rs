//! Commandes Tauri pour le module Search / Scraping multi-provider.

use crate::models::provider_contract::{ProviderCatalogEntry, ProviderCategory};
use crate::services::search::{catalog::SEARCH_PROVIDERS, test_connection};

#[tauri::command]
pub fn list_search_providers_catalog() -> Result<Vec<ProviderCatalogEntry>, String> {
    SEARCH_PROVIDERS
        .iter()
        .map(|provider| {
            let category = ProviderCategory::from_wire(provider.category)
                .ok_or_else(|| "Catalogue de fournisseurs indisponible".to_string())?;
            Ok(ProviderCatalogEntry::new(
                provider.id,
                provider.display_name,
                category,
                provider.signup_url,
                None,
                None,
            ))
        })
        .collect()
}

#[tauri::command]
pub async fn test_search_connection(provider_id: String) -> Result<(), String> {
    test_connection(&provider_id).await
}
