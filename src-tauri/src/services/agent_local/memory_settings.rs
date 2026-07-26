use super::memory_types::{MemoryMode, MemorySettings};
use std::path::{Path, PathBuf};

fn settings_path() -> PathBuf {
    crate::services::paths::data_dir().join("memory-settings.json")
}

pub async fn load() -> MemorySettings {
    load_from(&settings_path()).await
}

async fn load_from(path: &Path) -> MemorySettings {
    if !path.exists() {
        return MemorySettings::default();
    }
    match super::memory_io::read_bounded(path, 16 * 1024).await {
        Ok(data) => serde_json::from_str::<MemorySettings>(&data)
            .map(MemorySettings::normalized)
            .unwrap_or_default(),
        Err(_) => MemorySettings::default(),
    }
}

pub async fn save(settings: &MemorySettings) -> Result<MemorySettings, String> {
    let normalized = settings.clone().normalized();
    save_to(&settings_path(), &normalized).await?;
    Ok(normalized)
}

async fn save_to(path: &Path, settings: &MemorySettings) -> Result<(), String> {
    let data = serde_json::to_vec_pretty(settings).map_err(|error| {
        eprintln!("[memory] serialize settings: {error}");
        "Paramètres mémoire indisponibles.".to_string()
    })?;
    crate::services::private_store::atomic_write_async(path.to_path_buf(), data)
        .await
        .map_err(log_storage_error)
}

pub async fn set_mode(mode: MemoryMode) -> Result<MemorySettings, String> {
    let mut settings = load().await;
    settings.mode = mode;
    let saved = save(&settings).await?;
    if saved.mode.is_active() {
        super::memory_paths::MemoryLayout::production()
            .global_scope()
            .ensure()
            .await?;
    }
    Ok(saved)
}

pub async fn set_budget(tokens: u32) -> Result<MemorySettings, String> {
    let mut settings = load().await;
    settings.context_budget_tokens = tokens;
    save(&settings).await
}

fn log_storage_error(error: String) -> String {
    eprintln!("[memory] settings storage: {error}");
    "Paramètres mémoire indisponibles.".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn invalid_file_falls_back_to_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        tokio::fs::write(&path, b"{invalid").await.unwrap();

        assert_eq!(load_from(&path).await.mode, MemoryMode::Disabled);
    }

    #[tokio::test]
    async fn save_is_atomic_and_clamps_budget() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let settings = MemorySettings {
            mode: MemoryMode::Manual,
            context_budget_tokens: 99_999,
        }
        .normalized();

        save_to(&path, &settings).await.unwrap();
        let loaded = load_from(&path).await;

        assert_eq!(loaded.mode, MemoryMode::Manual);
        assert_eq!(loaded.context_budget_tokens, 3_000);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
