use super::types::{ExtensionContributions, ExtensionRecord, ExtensionStatus};

pub async fn disable_hosted_extensions() -> Result<bool, String> {
    let mut reminder = false;
    super::registry::mutate_state(|records, recovery_snapshot| {
        reminder = records.iter().any(|record| record.sensitive_access_granted);
        disable_records_preserving_snapshot(records, recovery_snapshot);
        Ok::<(), String>(())
    })?;
    crate::services::agent_local::permission_gate::clear_all_extensions().await;
    Ok(reminder)
}

pub fn recovery_snapshot_available() -> Result<bool, String> {
    super::registry_memory::snapshot().map(|state| state.recovery_snapshot.is_some())
}

pub async fn restore_recovery_snapshot() -> Result<bool, String> {
    let mut restored = false;
    super::registry::mutate_state(|records, recovery_snapshot| {
        restored = restore_records(records, recovery_snapshot.take());
        Ok::<(), String>(())
    })?;
    crate::services::agent_local::permission_gate::clear_all_extensions().await;
    Ok(restored)
}

pub(super) fn disable_hosted_records(records: &mut [ExtensionRecord]) {
    for record in records {
        record.enabled = false;
        record.status = ExtensionStatus::Inactive;
        record.last_error = None;
        record.contributions = ExtensionContributions::default();
    }
}

pub(crate) fn disable_records_and_snapshot(records: &mut [ExtensionRecord]) -> Vec<String> {
    let snapshot = records
        .iter()
        .filter(|record| record.enabled)
        .map(|record| record.manifest.id.clone())
        .collect();
    disable_hosted_records(records);
    snapshot
}

pub(crate) fn disable_records_preserving_snapshot(
    records: &mut [ExtensionRecord],
    recovery_snapshot: &mut Option<Vec<String>>,
) {
    if recovery_snapshot.is_none() {
        *recovery_snapshot = Some(disable_records_and_snapshot(records));
        return;
    }
    disable_hosted_records(records);
}

pub(crate) fn restore_records(
    records: &mut [ExtensionRecord],
    snapshot: Option<Vec<String>>,
) -> bool {
    let Some(ids) = snapshot else {
        return false;
    };
    let mut restored = false;
    for record in records {
        if ids.contains(&record.manifest.id) && record.trusted {
            record.enabled = true;
            record.status = ExtensionStatus::Inactive;
            record.last_error = None;
            restored = true;
        }
    }
    restored
}
