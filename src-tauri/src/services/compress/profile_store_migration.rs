use std::path::Path;

use super::profile_store::{write_document, CompressionProfileStoreError};
use super::profile_store_document::CompressionProfileDocument;
use super::profile_types::CompressionProfile;

pub(super) fn migrate(
    profile_path: &Path,
    config_path: &Path,
) -> Result<CompressionProfileDocument, CompressionProfileStoreError> {
    let seed: crate::services::config::LegacyCompressionSeed =
        crate::services::config::legacy_compression_seed_from_path(config_path)
            .map_err(|_| CompressionProfileStoreError::Migration)?;
    // La commande globale reste une autorite unique : la migration conserve
    // donc le choix explicite de l'utilisateur au lieu de le reinitialiser.
    let mut document = CompressionProfileDocument {
        automatic_enabled: seed.enabled.unwrap_or(true),
        ..CompressionProfileDocument::default()
    };
    document.profiles[0].threshold_percent = migrate_threshold(seed.threshold_percent);
    write_document(profile_path, &document)?;
    crate::services::config::finalize_compression_settings_migration_at(config_path)
        .map_err(|_| CompressionProfileStoreError::Migration)?;
    Ok(document)
}

pub(super) fn finish_existing(config_path: &Path) -> Result<bool, CompressionProfileStoreError> {
    crate::services::config::finalize_compression_settings_migration_at(config_path)
        .map_err(|_| CompressionProfileStoreError::Migration)
}

pub(super) fn acknowledge_backup(config_path: &Path) -> Result<(), CompressionProfileStoreError> {
    crate::services::config::acknowledge_compression_settings_backup_at(config_path)
        .map_err(|_| CompressionProfileStoreError::Migration)
}

pub(super) fn migrate_profile_v1(
    profile_path: &Path,
    original: &[u8],
    legacy: super::profile_store_v1::ProfileDocumentV1,
) -> Result<CompressionProfileDocument, CompressionProfileStoreError> {
    if legacy.schema_version != 1 {
        return Err(CompressionProfileStoreError::Invalid);
    }
    ensure_profile_backup(profile_path, original)?;
    let profiles = legacy.profiles.iter().filter_map(migrate_profile).collect();
    let mut document = CompressionProfileDocument {
        automatic_enabled: legacy.automatic_enabled,
        global_profile_id: legacy.global_profile_id,
        global_selection_revision: legacy.global_selection_revision.max(1),
        profiles,
        ..CompressionProfileDocument::default()
    };
    document.normalize();
    document
        .validate()
        .map_err(|_| CompressionProfileStoreError::Invalid)?;
    Ok(document)
}

pub(super) fn acknowledge_profile_backup(
    profile_path: &Path,
) -> Result<(), CompressionProfileStoreError> {
    let backup = profile_backup_path(profile_path)?;
    match crate::services::private_store::open_regular_single_link(&backup)
        .map_err(|_| CompressionProfileStoreError::Migration)?
    {
        Some(file) => {
            drop(file);
            std::fs::remove_file(backup).map_err(|_| CompressionProfileStoreError::Migration)
        }
        None => Ok(()),
    }
}

fn migrate_profile(value: &super::profile_store_v1::ProfileV1) -> Option<CompressionProfile> {
    if value.id.is_empty() || value.name.is_empty() {
        return None;
    }
    let mut profile = super::profile_defaults::beaver_profile();
    profile.id.clone_from(&value.id);
    profile.name.clone_from(&value.name);
    profile.revision = value.revision.max(1);
    profile.threshold_percent = value.threshold_percent.clamp(1, 90);
    profile.allow_under_64k = value.allow_under_64k;
    if let Some(prompt) = value.system_prompt() {
        profile.system_prompt = prompt.to_string();
    }
    if let Some(prompt) = value.handoff_prompt() {
        profile.handoff_prompt = prompt.to_string();
    }
    Some(profile)
}

fn ensure_profile_backup(
    profile_path: &Path,
    original: &[u8],
) -> Result<(), CompressionProfileStoreError> {
    let backup = profile_backup_path(profile_path)?;
    match crate::services::private_store::read_bounded_regular(&backup, 1024 * 1024)
        .map_err(|_| CompressionProfileStoreError::Migration)?
    {
        crate::services::private_store::BoundedFile::Missing => {
            crate::services::private_store::atomic_write(&backup, original)
                .map_err(|_| CompressionProfileStoreError::Migration)
        }
        crate::services::private_store::BoundedFile::Content(_) => Ok(()),
    }
}

fn profile_backup_path(
    profile_path: &Path,
) -> Result<std::path::PathBuf, CompressionProfileStoreError> {
    let stem = profile_path
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or(CompressionProfileStoreError::Migration)?;
    Ok(profile_path.with_file_name(format!("{stem}.v1.bak")))
}

fn migrate_threshold(value: Option<u8>) -> u8 {
    match value {
        None | Some(0) => 90,
        Some(value) => value.clamp(1, 90),
    }
}
