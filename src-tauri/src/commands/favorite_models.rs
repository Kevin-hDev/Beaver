//! Commandes Tauri pour la gestion des modèles favoris.

use crate::services::favorite_models::{self, FavoriteModel};

#[tauri::command]
pub fn list_favorite_models() -> Vec<FavoriteModel> {
    favorite_models::list()
}

#[tauri::command]
pub fn add_favorite_model(provider: String, model: String) -> Result<(), String> {
    favorite_models::add(&provider, &model)
}

#[tauri::command]
pub fn remove_favorite_model(provider: String, model: String) -> Result<(), String> {
    favorite_models::remove(&provider, &model)
}
