use std::path::Path;

use super::profile_store::{write_document, CompressionProfileStoreError};
use super::profile_store_document::CompressionProfileDocument;

pub(super) fn migrate(
    profile_path: &Path,
    config_path: &Path,
) -> Result<CompressionProfileDocument, CompressionProfileStoreError> {
    let seed: crate::services::config::LegacyCompressionSeed =
        crate::services::config::legacy_compression_seed_from_path(config_path)
            .map_err(|_| CompressionProfileStoreError::Migration)?;
    // The former global switch is intentionally not an authority in the new model.
    let _legacy_enabled = seed.enabled;
    let mut document = CompressionProfileDocument::default();
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

fn migrate_threshold(value: Option<u8>) -> u8 {
    match value {
        None | Some(0) => 90,
        Some(value) => value.clamp(1, 90),
    }
}
