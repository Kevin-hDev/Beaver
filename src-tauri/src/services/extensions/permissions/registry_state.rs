use super::types::{
    ExtensionContributions, ExtensionKind, ExtensionRecord, ExtensionStatus, ExtensionUiMode,
};
use std::collections::BTreeMap;

pub fn approve_local(record: &mut ExtensionRecord, trusted_at: &str) -> Result<(), String> {
    if record.kind != ExtensionKind::Local {
        return Err(super::error_codes::FINGERPRINT_FAILED.to_string());
    }
    record.fingerprint = Some(super::fingerprint::calculate(record)?);
    record.trusted_at = Some(trusted_at.to_string());
    record.trusted = true;
    Ok(())
}

pub fn revoke_fingerprints(
    records: &mut [ExtensionRecord],
    revocations: &BTreeMap<String, String>,
) -> bool {
    let mut reminder = false;
    for record in records
        .iter_mut()
        .filter(|record| revocations.contains_key(&record.manifest.id))
    {
        reminder |= record.sensitive_access_granted;
        record.trusted = false;
        record.trusted_at = None;
        record.enabled = false;
        record.status = ExtensionStatus::Error;
        record.last_error = revocations.get(&record.manifest.id).cloned();
        record.contributions = ExtensionContributions::default();
    }
    reminder
}

pub fn reset_hosted_runtime(mut records: Vec<ExtensionRecord>) -> Vec<ExtensionRecord> {
    for record in &mut records {
        let missing_advanced_artifact = record.kind == ExtensionKind::Local
            && record.ui_artifact.is_none()
            && record
                .manifest
                .ui
                .as_ref()
                .is_some_and(|ui| ui.mode == ExtensionUiMode::Advanced);
        if missing_advanced_artifact {
            record.enabled = false;
            record.trusted = false;
            record.trusted_at = None;
            record.status = ExtensionStatus::Error;
            record.last_error =
                Some(super::ui_contract::UI_DIAGNOSTIC_UI_ARTIFACT_INVALID.to_string());
        }
        if record.kind == ExtensionKind::Local && !record.trusted {
            record.enabled = false;
        }
        if !missing_advanced_artifact && !preserve_revocation_on_disable(record) {
            record.status = ExtensionStatus::Inactive;
            record.last_error = None;
        }
        record.contributions = ExtensionContributions::default();
    }
    records
}

pub fn preserve_revocation_on_disable(record: &ExtensionRecord) -> bool {
    record.last_error.as_deref().is_some_and(|error| {
        matches!(
            error,
            super::error_codes::FINGERPRINT_CHANGED | super::error_codes::FINGERPRINT_FAILED
        )
    })
}
