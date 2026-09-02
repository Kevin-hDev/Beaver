use super::types::{
    ExtensionKind, ExtensionRecord, ExtensionStatus, MAX_EXTENSIONS, MAX_MESSAGE_BYTES,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "extensions.json";
const VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoadedFormat {
    Missing,
    MigratedV0,
    V1,
}

#[derive(Debug)]
pub(crate) struct LoadedRegistry {
    pub extensions: Vec<ExtensionRecord>,
    pub recovery_snapshot: Option<Vec<String>>,
    pub format: LoadedFormat,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawEnvelope {
    version: u8,
    extensions: Vec<Value>,
    recovery_snapshot: Option<Vec<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Envelope<'a> {
    version: u8,
    extensions: &'a [ExtensionRecord],
    recovery_snapshot: &'a Option<Vec<String>>,
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
    if !path.exists() {
        return Ok(LoadedRegistry {
            extensions: Vec::new(),
            recovery_snapshot: None,
            format: LoadedFormat::Missing,
        });
    }
    let bytes = read_registry(path)?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| migration_error())?;
    match value {
        Value::Array(entries) => migrate_v0(path, &bytes, entries),
        Value::Object(_) => load_v1(value),
        _ => Err(migration_error()),
    }
}

fn read_registry(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = std::fs::metadata(path).map_err(|_| migration_error())?;
    if metadata.len() > MAX_MESSAGE_BYTES as u64 {
        return Err(migration_error());
    }
    let file = std::fs::File::open(path).map_err(|_| migration_error())?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_MESSAGE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| migration_error())?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(migration_error());
    }
    Ok(bytes)
}

fn migrate_v0(path: &Path, bytes: &[u8], entries: Vec<Value>) -> Result<LoadedRegistry, String> {
    if entries.len() > MAX_EXTENSIONS {
        return Err(migration_error());
    }
    crate::services::private_store::atomic_write(&v0_backup_path(path), bytes)
        .map_err(|_| migration_error())?;
    let mut extensions = parse_entries(entries)?;
    for record in extensions
        .iter_mut()
        .filter(|record| record.kind == ExtensionKind::Local)
    {
        record.trusted_at = None;
        record.sensitive_access_granted = false;
        match super::fingerprint::calculate(record) {
            Ok(fingerprint) => record.fingerprint = Some(fingerprint),
            Err(_) => {
                record.fingerprint = None;
                record.enabled = false;
                record.trusted = false;
                record.status = ExtensionStatus::Error;
                record.last_error = Some(super::error_codes::FINGERPRINT_FAILED.to_string());
            }
        }
    }
    save_to(path, &extensions, &None).map_err(|_| migration_error())?;
    Ok(LoadedRegistry {
        extensions,
        recovery_snapshot: None,
        format: LoadedFormat::MigratedV0,
    })
}

fn load_v1(value: Value) -> Result<LoadedRegistry, String> {
    let raw: RawEnvelope = serde_json::from_value(value).map_err(|_| migration_error())?;
    if raw.version != VERSION || raw.extensions.len() > MAX_EXTENSIONS {
        return Err(migration_error());
    }
    validate_recovery_snapshot(&raw.recovery_snapshot)?;
    let extensions = parse_entries(raw.extensions)?;
    Ok(LoadedRegistry {
        extensions,
        recovery_snapshot: raw.recovery_snapshot,
        format: LoadedFormat::V1,
    })
}

fn parse_entries(entries: Vec<Value>) -> Result<Vec<ExtensionRecord>, String> {
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
    if records.len() > MAX_EXTENSIONS {
        return Err("Trop d'extensions enregistrées.".to_string());
    }
    validate_recovery_snapshot(recovery_snapshot)?;
    let bytes = serde_json::to_vec_pretty(&Envelope {
        version: VERSION,
        extensions: records,
        recovery_snapshot,
    })
    .map_err(|_| "Registre d'extensions indisponible.".to_string())?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err("Registre d'extensions trop volumineux.".to_string());
    }
    crate::services::private_store::atomic_write(path, &bytes)
        .map_err(|_| "Registre d'extensions indisponible.".to_string())
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

pub(crate) fn finish_successful_startup(path: &Path, format: LoadedFormat) -> Result<(), String> {
    let backup = v0_backup_path(path);
    if format == LoadedFormat::V1 && backup.exists() {
        std::fs::remove_file(backup).map_err(|_| migration_error())?;
    }
    Ok(())
}

fn migration_error() -> String {
    super::error_codes::REGISTRY_MIGRATION_FAILED.to_string()
}
