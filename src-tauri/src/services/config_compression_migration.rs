use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LegacyCompressionSeed {
    pub threshold_percent: Option<u8>,
    pub enabled: Option<bool>,
}

pub(crate) fn legacy_compression_seed_from_path(
    path: &Path,
) -> Result<LegacyCompressionSeed, String> {
    let value = read_value(path)?;
    let advanced = value
        .get("advanced")
        .cloned()
        .and_then(|value| serde_json::from_value::<crate::models::AdvancedSettings>(value).ok())
        .unwrap_or_default();
    Ok(LegacyCompressionSeed {
        threshold_percent: advanced.legacy_compression_threshold,
        enabled: advanced.legacy_compression_enabled,
    })
}

pub(crate) fn finalize_compression_settings_migration_at(path: &Path) -> Result<bool, String> {
    let _guard = super::CONFIG_UPDATE_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(_) => return Err(migration_error()),
    };
    let mut value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| migration_error())?;
    let Some(advanced) = value
        .get_mut("advanced")
        .and_then(serde_json::Value::as_object_mut)
    else {
        return Ok(false);
    };
    let changed = advanced.remove("compression_enabled").is_some()
        | advanced.remove("compression_threshold").is_some();
    if !changed {
        return Ok(false);
    }
    ensure_backup(path, &bytes)?;
    let updated = serde_json::to_vec_pretty(&value).map_err(|_| migration_error())?;
    crate::services::private_store::atomic_write(path, &updated).map_err(|_| migration_error())?;
    Ok(true)
}

pub(crate) fn acknowledge_compression_settings_backup_at(path: &Path) -> Result<(), String> {
    let backup = backup_path(path)?;
    let Some(file) = crate::services::private_store::open_regular_single_link(&backup)
        .map_err(|_| migration_error())?
    else {
        return Ok(());
    };
    drop(file);
    std::fs::remove_file(backup).map_err(|_| migration_error())
}

fn read_value(path: &Path) -> Result<serde_json::Value, String> {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|_| migration_error()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(serde_json::Value::Object(serde_json::Map::new()))
        }
        Err(_) => Err(migration_error()),
    }
}

fn ensure_backup(path: &Path, original: &[u8]) -> Result<(), String> {
    let backup = backup_path(path)?;
    match crate::services::private_store::read_bounded_regular(&backup, 1024 * 1024)
        .map_err(|_| migration_error())?
    {
        crate::services::private_store::BoundedFile::Missing => {
            crate::services::private_store::atomic_write(&backup, original)
                .map_err(|_| migration_error())
        }
        crate::services::private_store::BoundedFile::Content(_) => Ok(()),
    }
}

fn backup_path(path: &Path) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| *name == "config.json")
        .ok_or_else(migration_error)?;
    Ok(path.with_file_name(format!("{name}.compression-v1.bak")))
}

fn migration_error() -> String {
    "Configuration indisponible.".to_string()
}
