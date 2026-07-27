use super::types::{ExtensionRecord, MAX_EXTENSIONS, MAX_MESSAGE_BYTES};
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "extensions.json";

pub fn path() -> PathBuf {
    crate::services::paths::data_dir().join(FILE_NAME)
}

pub fn load() -> Result<Vec<ExtensionRecord>, String> {
    load_from(&path())
}

pub fn save(records: &[ExtensionRecord]) -> Result<(), String> {
    save_to(&path(), records)
}

pub(crate) fn load_from(path: &Path) -> Result<Vec<ExtensionRecord>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let metadata =
        std::fs::metadata(path).map_err(|_| "Registre d'extensions indisponible.".to_string())?;
    if metadata.len() > MAX_MESSAGE_BYTES as u64 {
        return Err("Registre d'extensions invalide.".to_string());
    }
    let bytes =
        std::fs::read(path).map_err(|_| "Registre d'extensions indisponible.".to_string())?;
    let records: Vec<ExtensionRecord> = serde_json::from_slice(&bytes)
        .map_err(|_| "Registre d'extensions invalide.".to_string())?;
    if records.len() > MAX_EXTENSIONS {
        return Err("Trop d'extensions enregistrées.".to_string());
    }
    Ok(records)
}

pub(crate) fn save_to(path: &Path, records: &[ExtensionRecord]) -> Result<(), String> {
    if records.len() > MAX_EXTENSIONS {
        return Err("Trop d'extensions enregistrées.".to_string());
    }
    let bytes = serde_json::to_vec_pretty(records)
        .map_err(|_| "Registre d'extensions indisponible.".to_string())?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err("Registre d'extensions trop volumineux.".to_string());
    }
    crate::services::private_store::atomic_write(path, &bytes)
        .map_err(|_| "Registre d'extensions indisponible.".to_string())
}
