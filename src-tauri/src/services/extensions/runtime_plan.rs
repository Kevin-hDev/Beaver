use super::protocol::HostExtensionSpec;
use super::types::{ExtensionKind, ExtensionRecord};
use std::collections::BTreeMap;
use std::path::Path;

pub(super) struct HostPlan {
    pub official: Vec<ExtensionRecord>,
    pub third_party: Vec<ExtensionRecord>,
    pub failures: BTreeMap<String, String>,
}

pub(super) fn records(records: Vec<ExtensionRecord>) -> HostPlan {
    let mut official = Vec::new();
    let mut candidates = Vec::new();
    for record in records
        .into_iter()
        .filter(|record| record.enabled && record.trusted)
    {
        match record.kind {
            ExtensionKind::Builtin => official.push(record),
            ExtensionKind::Local => candidates.push(record),
        }
    }
    // Le dernier emplacement reste réservé à l'autorité officielle afin que le
    // plafond tiers soit stable même quand aucun Builtin n'est actif.
    let capacity = super::types::MAX_HOST_PROCESSES.saturating_sub(1);
    let mut failures = BTreeMap::new();
    for record in candidates.iter().skip(capacity) {
        failures.insert(
            record.manifest.id.clone(),
            super::error_codes::LIMIT_REACHED.to_string(),
        );
    }
    candidates.truncate(capacity);
    HostPlan {
        official,
        third_party: candidates,
        failures,
    }
}

pub(super) fn specification(
    record: ExtensionRecord,
    host_directory: &Path,
) -> Option<HostExtensionSpec> {
    let main = match record.kind {
        ExtensionKind::Builtin => super::builtin::resolve_entry(host_directory, &record),
        ExtensionKind::Local => super::manifest::resolve_record_entry(&record),
    }
    .ok()?;
    Some(HostExtensionSpec {
        id: record.manifest.id.clone(),
        main_path: main.to_str()?.to_string(),
        manifest: record.manifest,
    })
}
