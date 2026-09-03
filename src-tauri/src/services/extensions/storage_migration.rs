use super::storage::{LoadedFormat, LoadedRegistry};
use super::types::{ExtensionKind, ExtensionStatus, MAX_EXTENSIONS};
use serde_json::Value;
use std::path::Path;

pub(super) fn migrate_v1(
    path: &Path,
    source_bytes: &[u8],
    mut source: Value,
) -> Result<LoadedRegistry, String> {
    let entries = source
        .get_mut("extensions")
        .and_then(Value::as_array_mut)
        .ok_or_else(super::storage::migration_error)?;
    if entries.len() > MAX_EXTENSIONS {
        return Err(super::storage::migration_error());
    }
    for entry in entries {
        let Some(manifest) = entry.get_mut("manifest").and_then(Value::as_object_mut) else {
            return Err(super::storage::migration_error());
        };
        if let Some(legacy) = manifest
            .get("ui")
            .and_then(Value::as_str)
            .map(str::to_string)
        {
            manifest.insert("ui".to_string(), Value::Null);
            manifest.insert("uiLegacy".to_string(), Value::String(legacy));
        }
    }
    source["version"] = Value::from(2_u8);
    let mut loaded = super::storage::load_v2(source.clone())?;
    let migrated = super::storage_format::serialize_value(&source)
        .map_err(|_| super::storage::migration_error())?;
    // La sauvegarde complète précède la publication v2 : une interruption ne
    // peut jamais laisser une migration sans point de reprise exact.
    crate::services::private_store::atomic_write(
        &super::storage::v1_backup_path(path),
        source_bytes,
    )
    .map_err(|_| super::storage::migration_error())?;
    crate::services::private_store::atomic_write(path, &migrated)
        .map_err(|_| super::storage::migration_error())?;
    loaded.format = LoadedFormat::MigratedV1;
    Ok(loaded)
}

pub(super) fn migrate_v0(
    path: &Path,
    source_bytes: &[u8],
    entries: Vec<Value>,
) -> Result<LoadedRegistry, String> {
    if entries.len() > MAX_EXTENSIONS {
        return Err(super::storage::migration_error());
    }
    let mut extensions = super::storage::parse_entries(entries)?;
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
    // La projection v2 est validée avant la sauvegarde : une migration refusée
    // ne publie jamais une sauvegarde qui donnerait l'illusion d'avoir réussi.
    let migrated = super::storage::serialize_envelope(&extensions, &None)
        .map_err(|_| super::storage::migration_error())?;
    crate::services::private_store::atomic_write(
        &super::storage::v0_backup_path(path),
        source_bytes,
    )
    .map_err(|_| super::storage::migration_error())?;
    crate::services::private_store::atomic_write(path, &migrated)
        .map_err(|_| super::storage::migration_error())?;
    Ok(LoadedRegistry {
        extensions,
        recovery_snapshot: None,
        format: LoadedFormat::MigratedV0,
    })
}
