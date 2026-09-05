use serde::Serialize;
#[cfg(test)]
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionRecoveryState {
    pub extension_id: Option<String>,
    pub stage: Option<String>,
    pub attempts: Option<u8>,
    pub can_retry: bool,
    pub marker_invalid: bool,
    pub recovery_snapshot_available: bool,
}

pub fn state() -> Result<ExtensionRecoveryState, String> {
    let recovery_snapshot_available = super::registry_recovery::recovery_snapshot_available()?;
    let records = super::registry::list()?;
    Ok(from_marker(
        classify_marker(super::loading_marker::read(), &records),
        recovery_snapshot_available,
    ))
}

#[cfg(test)]
pub(crate) fn state_at(path: &Path, snapshot_available: bool) -> ExtensionRecoveryState {
    from_marker(super::loading_marker::read_at(path), snapshot_available)
}

#[cfg(test)]
pub(crate) fn state_at_with_records(
    path: &Path,
    snapshot_available: bool,
    records: &[super::types::ExtensionRecord],
) -> ExtensionRecoveryState {
    from_marker(
        classify_marker(super::loading_marker::read_at(path), records),
        snapshot_available,
    )
}

pub async fn keep_disabled(extension_id: &str) -> Result<bool, String> {
    require_marker(extension_id)?;
    let reminder = super::registry::set_enabled(extension_id, false, false).await?;
    super::loading_marker::discard()?;
    Ok(reminder)
}

pub async fn retry(extension_id: &str) -> Result<bool, String> {
    super::validation::identifier(extension_id)?;
    let attempts = super::loading_marker::next_retry_attempt(extension_id)?;
    super::runtime_lifecycle::retry_load(extension_id.to_string(), attempts).await
}

pub fn discard_marker() -> Result<(), String> {
    let records = super::registry::list()?;
    discard_classified_marker(
        classify_marker(super::loading_marker::read(), &records),
        super::loading_marker::discard,
    )
}

#[cfg(test)]
pub(crate) fn discard_marker_at(
    path: &Path,
    records: &[super::types::ExtensionRecord],
) -> Result<(), String> {
    discard_classified_marker(
        classify_marker(super::loading_marker::read_at(path), records),
        || super::loading_marker::discard_at(path),
    )
}

fn require_marker(extension_id: &str) -> Result<(), String> {
    super::validation::identifier(extension_id)?;
    match super::loading_marker::read() {
        super::loading_marker::MarkerRead::Valid(marker) if marker.extension_id == extension_id => {
            Ok(())
        }
        _ => Err(super::error_codes::RECOVERY_MARKER_INVALID.to_string()),
    }
}

fn classify_marker(
    marker: super::loading_marker::MarkerRead,
    records: &[super::types::ExtensionRecord],
) -> super::loading_marker::MarkerRead {
    match marker {
        super::loading_marker::MarkerRead::Valid(marker)
            if !records
                .iter()
                .any(|record| record.manifest.id == marker.extension_id) =>
        {
            super::loading_marker::MarkerRead::Invalid
        }
        marker => marker,
    }
}

fn discard_classified_marker(
    marker: super::loading_marker::MarkerRead,
    discard: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    if !matches!(marker, super::loading_marker::MarkerRead::Invalid) {
        return Err(super::error_codes::RECOVERY_MARKER_INVALID.to_string());
    }
    discard()
}

fn from_marker(
    marker: super::loading_marker::MarkerRead,
    recovery_snapshot_available: bool,
) -> ExtensionRecoveryState {
    match marker {
        super::loading_marker::MarkerRead::Missing => ExtensionRecoveryState {
            extension_id: None,
            stage: None,
            attempts: None,
            can_retry: false,
            marker_invalid: false,
            recovery_snapshot_available,
        },
        super::loading_marker::MarkerRead::Valid(marker) => {
            let can_retry = marker.can_retry();
            ExtensionRecoveryState {
                extension_id: Some(marker.extension_id),
                stage: Some(marker.stage),
                attempts: Some(marker.attempts),
                can_retry,
                marker_invalid: false,
                recovery_snapshot_available,
            }
        }
        super::loading_marker::MarkerRead::Invalid => ExtensionRecoveryState {
            extension_id: None,
            stage: None,
            attempts: None,
            can_retry: false,
            marker_invalid: true,
            recovery_snapshot_available,
        },
    }
}
