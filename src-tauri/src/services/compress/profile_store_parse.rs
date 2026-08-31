use std::path::Path;

use super::profile_store::CompressionProfileStoreError;
use super::profile_store_document::CompressionProfileDocument;

pub(super) fn parse_document(
    profile_path: &Path,
    bytes: &[u8],
) -> Result<(CompressionProfileDocument, bool), CompressionProfileStoreError> {
    let mut value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| CompressionProfileStoreError::Invalid)?;
    let version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .and_then(|number| u16::try_from(number).ok())
        .unwrap_or(1);
    if version == 1 {
        let legacy =
            serde_json::from_value(value).map_err(|_| CompressionProfileStoreError::Invalid)?;
        return super::profile_store_migration::migrate_profile_v1(profile_path, bytes, legacy)
            .map(|document| (document, true));
    }
    if version > super::profile_store_document::PROFILE_SCHEMA_VERSION {
        log::warn!("compression_profile_document_future_version version={version}");
        return Err(CompressionProfileStoreError::FutureVersion(version));
    }
    if version != super::profile_store_document::PROFILE_SCHEMA_VERSION {
        return Err(CompressionProfileStoreError::Invalid);
    }
    merge_defaults(&mut value)?;
    serde_json::from_value(value)
        .map(|document| (document, false))
        .map_err(|_| CompressionProfileStoreError::Invalid)
}

fn merge_defaults(value: &mut serde_json::Value) -> Result<(), CompressionProfileStoreError> {
    let defaults = serde_json::to_value(CompressionProfileDocument::default())
        .map_err(|_| CompressionProfileStoreError::Invalid)?;
    merge_missing_fields(value, &defaults);
    let profile_defaults = serde_json::to_value(super::profile_defaults::beaver_profile())
        .map_err(|_| CompressionProfileStoreError::Invalid)?;
    if let Some(profiles) = value
        .get_mut("profiles")
        .and_then(serde_json::Value::as_array_mut)
    {
        profiles.truncate(super::profile_limits::MAX_PROFILE_READ_CANDIDATES);
        for profile in profiles.iter_mut() {
            merge_missing_fields(profile, &profile_defaults);
        }
    }
    Ok(())
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
