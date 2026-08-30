use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::profile_store_document::CompressionProfileDocument;

const MAX_PROFILE_DOCUMENT_BYTES: u64 = 1024 * 1024;
const MAX_TRACKED_MIGRATIONS: usize = 64;
static PROFILE_STORE_LOCK: Mutex<()> = Mutex::new(());
static MIGRATED_PROFILE_PATHS: std::sync::LazyLock<Mutex<HashSet<PathBuf>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashSet::new()));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionProfileStoreError {
    Read,
    Invalid,
    FutureVersion(u16),
    Write,
    Migration,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalCompressionTriggerSettings {
    pub threshold_percent: u8,
    pub available: bool,
}

pub fn load_document() -> Result<CompressionProfileDocument, CompressionProfileStoreError> {
    let _guard = PROFILE_STORE_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    load_from_paths(&profile_path(), &crate::services::config::config_path())
}

pub fn mutate_document<T>(
    mutation: impl FnOnce(&mut CompressionProfileDocument) -> Result<T, CompressionProfileStoreError>,
) -> Result<(T, CompressionProfileDocument), CompressionProfileStoreError> {
    let _guard = PROFILE_STORE_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let path = profile_path();
    let mut document = load_from_paths(&path, &crate::services::config::config_path())?;
    let result = mutation(&mut document)?;
    document
        .validate()
        .map_err(|_| CompressionProfileStoreError::Invalid)?;
    write_document(&path, &document)?;
    Ok((result, document))
}

#[cfg(test)]
pub(crate) fn trigger_settings(
    document: &CompressionProfileDocument,
    context_window: u64,
) -> Result<GlobalCompressionTriggerSettings, CompressionProfileStoreError> {
    let profile = document
        .profiles
        .iter()
        .find(|profile| profile.id == document.global_profile_id)
        .ok_or(CompressionProfileStoreError::Invalid)?;
    let band_available = match super::profile_budget::band_for_window(context_window) {
        Some(super::profile_types::CompressionWindowBand::Under64K) => profile.allow_under_64k,
        Some(_) | None => true,
    };
    Ok(GlobalCompressionTriggerSettings {
        threshold_percent: profile.threshold_percent,
        available: document.automatic_enabled && band_available,
    })
}

pub(crate) fn load_from_paths(
    profile_path: &Path,
    config_path: &Path,
) -> Result<CompressionProfileDocument, CompressionProfileStoreError> {
    match crate::services::private_store::read_bounded_regular(
        profile_path,
        MAX_PROFILE_DOCUMENT_BYTES,
    )
    .map_err(|_| CompressionProfileStoreError::Read)?
    {
        crate::services::private_store::BoundedFile::Missing => {
            let document = super::profile_store_migration::migrate(profile_path, config_path)?;
            remember_migration(profile_path);
            Ok(document)
        }
        crate::services::private_store::BoundedFile::Content(bytes) => {
            let mut document = match parse_document(&bytes) {
                Ok(document) => document,
                Err(_) => {
                    ::log::warn!("compression_profile_document_invalid_json");
                    let document = CompressionProfileDocument::default();
                    write_document(profile_path, &document)?;
                    document
                }
            };
            let before = document.clone();
            normalize_loaded_document(&mut document)?;
            if document != before {
                write_document(profile_path, &document)?;
            }
            let migrated_now = super::profile_store_migration::finish_existing(config_path)?;
            if migrated_now {
                remember_migration(profile_path);
            } else if !migrated_in_this_process(profile_path)
                && super::profile_store_migration::acknowledge_backup(config_path).is_err()
            {
                log::warn!("compression_profile_migration_backup_cleanup_failed");
            }
            Ok(document)
        }
    }
}

fn parse_document(bytes: &[u8]) -> Result<CompressionProfileDocument, serde_json::Error> {
    let mut value: serde_json::Value = serde_json::from_slice(bytes)?;
    let defaults = serde_json::to_value(CompressionProfileDocument::default())?;
    merge_missing_fields(&mut value, &defaults);

    let profile_defaults = serde_json::to_value(super::profile_defaults::beaver_profile())?;
    if let Some(profiles) = value
        .get_mut("profiles")
        .and_then(serde_json::Value::as_array_mut)
    {
        for profile in profiles {
            merge_missing_fields(profile, &profile_defaults);
        }
    }
    serde_json::from_value(value)
}

fn merge_missing_fields(value: &mut serde_json::Value, defaults: &serde_json::Value) {
    let (Some(value), Some(defaults)) = (value.as_object_mut(), defaults.as_object()) else {
        return;
    };
    for (key, default) in defaults {
        match value.get_mut(key) {
            Some(existing) => merge_missing_fields(existing, default),
            None => {
                value.insert(key.clone(), default.clone());
            }
        }
    }
}

pub(super) fn write_document(
    path: &Path,
    document: &CompressionProfileDocument,
) -> Result<(), CompressionProfileStoreError> {
    let mut normalized = document.clone();
    normalize_loaded_document(&mut normalized)?;
    let bytes = serde_json::to_vec_pretty(&normalized)
        .map_err(|_| CompressionProfileStoreError::Invalid)?;
    if bytes.len() > MAX_PROFILE_DOCUMENT_BYTES as usize {
        return Err(CompressionProfileStoreError::Invalid);
    }
    crate::services::private_store::atomic_write(path, &bytes)
        .map_err(|_| CompressionProfileStoreError::Write)
}

fn normalize_loaded_document(
    document: &mut CompressionProfileDocument,
) -> Result<(), CompressionProfileStoreError> {
    if document.schema_version != super::profile_store_document::PROFILE_SCHEMA_VERSION {
        if document.schema_version > super::profile_store_document::PROFILE_SCHEMA_VERSION {
            log::warn!(
                "compression_profile_document_future_version version={}",
                document.schema_version
            );
            return Err(CompressionProfileStoreError::FutureVersion(
                document.schema_version,
            ));
        }
        return Err(CompressionProfileStoreError::Invalid);
    }
    document.normalize();
    document
        .validate()
        .map_err(|_| CompressionProfileStoreError::Invalid)
}

fn profile_path() -> PathBuf {
    crate::services::paths::data_dir().join("compression-profiles.json")
}

fn remember_migration(path: &Path) {
    let mut migrated = MIGRATED_PROFILE_PATHS
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if migrated.len() >= MAX_TRACKED_MIGRATIONS && !migrated.contains(path) {
        if let Some(oldest) = migrated.iter().next().cloned() {
            migrated.remove(&oldest);
        }
    }
    migrated.insert(path.to_path_buf());
}

fn migrated_in_this_process(path: &Path) -> bool {
    let migrated = MIGRATED_PROFILE_PATHS
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    migrated.contains(path)
}

#[cfg(test)]
pub(crate) fn forget_migration_marker_for_test(path: &Path) {
    let mut migrated = MIGRATED_PROFILE_PATHS
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    migrated.remove(path);
}

#[cfg(test)]
pub(crate) fn save_to_path_fail_before_replace(
    path: &Path,
    document: &CompressionProfileDocument,
) -> Result<(), CompressionProfileStoreError> {
    let bytes =
        serde_json::to_vec_pretty(document).map_err(|_| CompressionProfileStoreError::Invalid)?;
    crate::services::private_store::atomic_write_fail_before_replace(path, &bytes)
        .map_err(|_| CompressionProfileStoreError::Write)
}
