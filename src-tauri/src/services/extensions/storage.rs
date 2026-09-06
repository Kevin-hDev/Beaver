use super::types::{ExtensionRecord, MAX_EXTENSIONS, MAX_MESSAGE_BYTES};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "extensions.json";
const VERSION: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoadedFormat {
    Missing,
    MigratedV0,
    MigratedV1,
    V2,
}

#[derive(Debug)]
pub(crate) struct LoadedRegistry {
    pub extensions: Vec<ExtensionRecord>,
    pub recovery_snapshot: Option<Vec<String>>,
    pub format: LoadedFormat,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawEnvelope {
    version: u8,
    extensions: Vec<Value>,
    recovery_snapshot: Option<Vec<String>>,
}

pub fn path() -> PathBuf {
    crate::services::paths::data_dir().join(FILE_NAME)
}

pub fn load() -> Result<LoadedRegistry, String> {
    load_from(&path())
}

pub fn save(
    records: &[ExtensionRecord],
    recovery_snapshot: &Option<Vec<String>>,
) -> Result<(), String> {
    save_to(&path(), records, recovery_snapshot)
}

pub(crate) fn load_from(path: &Path) -> Result<LoadedRegistry, String> {
    if !registry_exists(path)? {
        return Ok(LoadedRegistry {
            extensions: Vec::new(),
            recovery_snapshot: None,
            format: LoadedFormat::Missing,
        });
    }
    let bytes = read_registry(path)?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| migration_error())?;
    match value {
        Value::Array(entries) => super::storage_migration::migrate_v0(path, &bytes, entries),
        Value::Object(_) => match value.get("version").and_then(Value::as_u64) {
            Some(1) => super::storage_migration::migrate_v1(path, &bytes, value),
            Some(2) => load_v2(value),
            Some(version) if version > u64::from(VERSION) => {
                Err(super::error_codes::REGISTRY_VERSION_UNSUPPORTED.to_string())
            }
            _ => Err(migration_error()),
        },
        _ => Err(migration_error()),
    }
}

fn read_registry(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = std::fs::metadata(path).map_err(|_| unavailable_error())?;
    if metadata.len() > MAX_MESSAGE_BYTES as u64 {
        return Err(migration_error());
    }
    let file = std::fs::File::open(path).map_err(|_| unavailable_error())?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_MESSAGE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| unavailable_error())?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(migration_error());
    }
    Ok(bytes)
}

pub(super) fn load_v2(value: Value) -> Result<LoadedRegistry, String> {
    let raw: RawEnvelope = serde_json::from_value(value).map_err(|_| migration_error())?;
    if raw.version != VERSION || raw.extensions.len() > MAX_EXTENSIONS {
        return Err(migration_error());
    }
    validate_recovery_snapshot(&raw.recovery_snapshot)?;
    let extensions = parse_entries(raw.extensions)?;
    validate_loaded_records(&extensions)?;
    Ok(LoadedRegistry {
        extensions,
        recovery_snapshot: raw.recovery_snapshot,
        format: LoadedFormat::V2,
    })
}

pub(super) fn parse_entries(entries: Vec<Value>) -> Result<Vec<ExtensionRecord>, String> {
    let mut records = Vec::with_capacity(entries.len());
    for entry in entries {
        let supported = matches!(
            entry.get("kind").and_then(Value::as_str),
            Some("builtin" | "local")
        );
        if !supported {
            ::log::warn!(
                "[extensions] {}",
                super::error_codes::REGISTRY_ENTRY_IGNORED
            );
            continue;
        }
        records.push(serde_json::from_value(entry).map_err(|_| migration_error())?);
    }
    Ok(records)
}

pub(crate) fn save_to(
    path: &Path,
    records: &[ExtensionRecord],
    recovery_snapshot: &Option<Vec<String>>,
) -> Result<(), String> {
    validate_recovery_snapshot(recovery_snapshot)?;
    let existing = if registry_exists(path)? {
        let bytes = read_registry(path)?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|_| migration_error())?;
        if value
            .get("version")
            .and_then(Value::as_u64)
            .is_some_and(|version| version > u64::from(VERSION))
        {
            return Err(super::error_codes::REGISTRY_VERSION_UNSUPPORTED.to_string());
        }
        // Never let a mutation repair or downgrade a refused registry.
        load_v2(value.clone())?;
        Some(value)
    } else {
        None
    };
    let bytes = super::storage_format::serialize(VERSION, records, recovery_snapshot, existing)?;
    crate::services::private_store::atomic_write(path, &bytes).map_err(|_| unavailable_error())
}

pub(super) fn serialize_envelope(
    records: &[ExtensionRecord],
    recovery_snapshot: &Option<Vec<String>>,
) -> Result<Vec<u8>, String> {
    validate_recovery_snapshot(recovery_snapshot)?;
    super::storage_format::serialize(VERSION, records, recovery_snapshot, None)
}

fn validate_recovery_snapshot(snapshot: &Option<Vec<String>>) -> Result<(), String> {
    let Some(ids) = snapshot else {
        return Ok(());
    };
    if ids.len() > MAX_EXTENSIONS {
        return Err(super::error_codes::RECOVERY_MARKER_INVALID.to_string());
    }
    let mut unique = HashSet::with_capacity(ids.len());
    if ids
        .iter()
        .any(|id| super::validation::identifier(id).is_err() || !unique.insert(id.as_str()))
    {
        return Err(super::error_codes::RECOVERY_MARKER_INVALID.to_string());
    }
    Ok(())
}

pub(crate) fn v0_backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.v0.bak")
}

pub(crate) fn v1_backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.v1.bak")
}

pub(crate) fn finish_successful_startup(path: &Path, format: LoadedFormat) -> Result<(), String> {
    if format == LoadedFormat::V2 {
        for backup in [v0_backup_path(path), v1_backup_path(path)] {
            if backup.exists() {
                std::fs::remove_file(backup).map_err(|_| migration_error())?;
            }
        }
    }
    Ok(())
}

pub(super) fn migration_error() -> String {
    super::error_codes::REGISTRY_MIGRATION_FAILED.to_string()
}

fn registry_exists(path: &Path) -> Result<bool, String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(unavailable_error()),
    }
}

fn unavailable_error() -> String {
    super::error_codes::REGISTRY_UNAVAILABLE.to_string()
}

pub(super) fn validate_loaded_records(records: &[ExtensionRecord]) -> Result<(), String> {
    // Apply the same runtime reset as startup before validating legacy records.
    let records = super::registry_state::reset_hosted_runtime(records.to_vec());
    super::validation::records(&records).map_err(|_| migration_error())
}
