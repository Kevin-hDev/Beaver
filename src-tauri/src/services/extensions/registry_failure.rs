use super::host_identity::HostIdentity;
use super::types::{ExtensionContributions, ExtensionKind, ExtensionRecord, ExtensionStatus};

pub(super) fn mark_identity_error(identity: &HostIdentity) -> Vec<String> {
    mark_identity_failure(identity, identity_error_code(), false)
}

pub(super) fn identity_error_code() -> &'static str {
    super::error_codes::HOST_UNAVAILABLE
}

pub(super) fn mark_identity_stop_unconfirmed(identity: &HostIdentity) -> Vec<String> {
    mark_identity_failure(identity, super::error_codes::STOP_UNCONFIRMED, true)
}

fn mark_identity_failure(identity: &HostIdentity, error: &str, disable: bool) -> Vec<String> {
    let mut affected = Vec::new();
    let _ = super::registry::mutate(|records| {
        apply_identity_failure(records, identity, error, disable, &mut affected);
        Ok::<(), String>(())
    });
    affected
}

pub(super) fn apply_identity_failure(
    records: &mut [ExtensionRecord],
    identity: &HostIdentity,
    error: &str,
    disable: bool,
    affected: &mut Vec<String>,
) {
    for record in records.iter_mut().filter(|record| {
        let same_host = match identity {
            HostIdentity::Official => record.kind == ExtensionKind::Builtin,
            HostIdentity::ThirdParty(id) => record.manifest.id == *id,
        };
        same_host && (disable || record.enabled)
    }) {
        if affected.len() < super::types::MAX_EXTENSIONS {
            affected.push(record.manifest.id.clone());
        }
        if disable {
            record.enabled = false;
        }
        record.status = ExtensionStatus::Error;
        record.last_error = Some(error.to_string());
        record.contributions = ExtensionContributions::default();
    }
}
