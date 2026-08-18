//! Persistence des modèles favoris dans un fichier local.
//! Format : `[{"provider": "openrouter", "model": "qwen/qwen3-235b"}, ...]`

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FavoriteModel {
    pub provider: String,
    pub model: String,
}

fn favorites_path() -> PathBuf {
    crate::services::paths::data_dir().join("favorite-models.json")
}

pub fn list() -> Vec<FavoriteModel> {
    let path = favorites_path();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn add(provider: &str, model: &str) -> Result<(), String> {
    let mut favs = list();
    let entry = FavoriteModel {
        provider: provider.to_string(),
        model: model.to_string(),
    };
    if favs.contains(&entry) {
        return Ok(());
    }
    favs.push(entry);
    write_atomic(&favs)
}

pub fn remove(provider: &str, model: &str) -> Result<(), String> {
    let mut favs = list();
    let before = favs.len();
    favs.retain(|f| !(f.provider == provider && f.model == model));
    if favs.len() == before {
        return Ok(());
    }
    write_atomic(&favs)
}

fn write_atomic(favs: &[FavoriteModel]) -> Result<(), String> {
    let path = favorites_path();
    let json = serde_json::to_string_pretty(favs).map_err(|e| format!("json: {e}"))?;
    crate::services::private_store::atomic_write(&path, json.as_bytes())
        .map_err(|_| "favorite-models-write".to_string())
}
