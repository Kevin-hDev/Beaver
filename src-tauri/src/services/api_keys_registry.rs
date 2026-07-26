fn registry_path() -> std::path::PathBuf {
    crate::services::paths::data_dir().join("configured-providers.json")
}

fn provider_ids<'a>(keys: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut ids: Vec<String> = keys
        .filter(|id| validate::validate_provider(id).is_ok())
        .map(str::to_string)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

fn write_registry(ids: &[String]) -> Result<(), String> {
    let path = registry_path();
    let json = serde_json::to_string_pretty(ids).map_err(|_| "erreur de stockage".to_string())?;
    crate::services::private_store::atomic_write(&path, json.as_bytes())
}

fn configured_from_state() -> Vec<String> {
    let Ok(state) = STATE.lock() else {
        return Vec::new();
    };
    let Some(current) = state.as_ref() else {
        return Vec::new();
    };
    provider_ids(current.keys.keys().map(String::as_str))
}

fn sync_registry_cache() {
    let ids = configured_from_state();
    if write_registry(&ids).is_err() {
        eprintln!("[vault] provider registry synchronization failed");
    }
}
