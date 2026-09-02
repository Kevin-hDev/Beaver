use super::call_context::ExtensionCallContext;
use super::host_identity::HostIdentity;
use super::types::{ExtensionApiLevel, ExtensionKind, ExtensionRecord};

pub(super) fn authorize_call(context: &ExtensionCallContext) -> Result<bool, String> {
    debug_assert!(context.generation() > 0);
    super::registry_memory::with_records(|records| authorized_records(records, context))
        .map_err(|_| unavailable())
}

pub(super) fn authorized_records(
    records: &[ExtensionRecord],
    context: &ExtensionCallContext,
) -> bool {
    match context.identity() {
        HostIdentity::ThirdParty(id) => records.iter().any(|record| {
            record.kind == ExtensionKind::Local
                && record.manifest.id == *id
                && record.enabled
                && record.trusted
                && record.manifest.api_level == *context.api_level()
        }),
        HostIdentity::Official => {
            official_level(records).is_some_and(|level| level == *context.api_level())
        }
    }
}

pub(super) fn mark_sensitive_access(identity: &HostIdentity) -> Result<(), String> {
    if let HostIdentity::ThirdParty(id) = identity {
        super::validation::identifier(id)?;
    }
    if super::registry_memory::with_records(|records| {
        sensitive_identity_is_marked(records, identity)
    })? {
        return Ok(());
    }
    super::registry::mutate(|records| {
        mark_sensitive_identity(records, identity)
            .then_some(())
            .ok_or_else(unavailable)
    })
}

pub(super) fn sensitive_identity_is_marked(
    records: &[ExtensionRecord],
    identity: &HostIdentity,
) -> bool {
    let matching = records
        .iter()
        .filter(|record| matches_identity(record, identity));
    let mut found = false;
    for record in matching {
        found = true;
        if !record.sensitive_access_granted {
            return false;
        }
    }
    found
}

pub(super) fn mark_sensitive_identity(
    records: &mut [ExtensionRecord],
    identity: &HostIdentity,
) -> bool {
    let mut marked = false;
    for record in records
        .iter_mut()
        .filter(|record| matches_identity(record, identity))
    {
        record.sensitive_access_granted = true;
        marked = true;
    }
    marked
}

fn matches_identity(record: &ExtensionRecord, identity: &HostIdentity) -> bool {
    record.enabled
        && record.trusted
        && match identity {
            HostIdentity::Official => record.kind == ExtensionKind::Builtin,
            HostIdentity::ThirdParty(id) => {
                record.kind == ExtensionKind::Local && record.manifest.id == *id
            }
        }
}

fn official_level(records: &[ExtensionRecord]) -> Option<ExtensionApiLevel> {
    let active = records
        .iter()
        .filter(|record| record.kind == ExtensionKind::Builtin && record.enabled && record.trusted);
    if active
        .clone()
        .any(|record| record.manifest.api_level == ExtensionApiLevel::Advanced)
    {
        Some(ExtensionApiLevel::Advanced)
    } else if active.count() > 0 {
        Some(ExtensionApiLevel::Stable)
    } else {
        None
    }
}

fn unavailable() -> String {
    "Registre d'extensions indisponible.".to_string()
}
