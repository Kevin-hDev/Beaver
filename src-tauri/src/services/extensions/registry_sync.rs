use super::types::{ExtensionContributions, ExtensionKind, ExtensionStatus, MAX_TOOLS};
use std::collections::{BTreeMap, HashMap, HashSet};

pub fn mark_enabled_loading() -> Result<(), String> {
    super::registry::mutate(|records| {
        for record in records.iter_mut().filter(|record| {
            record.kind != ExtensionKind::External && record.enabled && record.trusted
        }) {
            record.status = ExtensionStatus::Loading;
            record.last_error = None;
        }
        Ok::<(), String>(())
    })
}

pub fn apply_results(
    enabled_ids: &HashSet<String>,
    successful: HashMap<String, ExtensionContributions>,
    failures: &BTreeMap<String, String>,
) -> Result<usize, String> {
    let total_tools = successful
        .values()
        .try_fold(0_usize, |total, contributions| {
            super::validation::contributions(&contributions.tools, &contributions.events)?;
            Ok::<usize, String>(total.saturating_add(contributions.tools.len()))
        })?;
    if total_tools > MAX_TOOLS {
        return Err("Nombre maximal d'outils d'extension atteint.".to_string());
    }
    let active = successful.len();
    super::registry::mutate(move |records| {
        for record in records
            .iter_mut()
            .filter(|record| enabled_ids.contains(&record.manifest.id))
        {
            if let Some(contributions) = successful.get(&record.manifest.id) {
                record.contributions = contributions.clone();
                record.status = ExtensionStatus::Active;
                record.last_error = None;
            } else {
                record.contributions = ExtensionContributions::default();
                record.status = ExtensionStatus::Error;
                record.last_error = Some(
                    failures
                        .get(&record.manifest.id)
                        .cloned()
                        .unwrap_or_else(|| "load_failed".to_string()),
                );
            }
        }
        Ok::<(), String>(())
    })?;
    Ok(active)
}

pub fn mark_all_enabled_error() {
    let _ = super::registry::mutate(|records| {
        for record in records
            .iter_mut()
            .filter(|record| record.kind != ExtensionKind::External && record.enabled)
        {
            record.status = ExtensionStatus::Error;
            record.last_error = Some("host_unavailable".to_string());
        }
        Ok::<(), String>(())
    });
}

pub fn mark_identity_error(identity: &super::host_identity::HostIdentity) {
    let _ = super::registry::mutate(|records| {
        for record in records.iter_mut().filter(|record| match identity {
            super::host_identity::HostIdentity::Official => {
                record.kind == ExtensionKind::Builtin && record.enabled
            }
            super::host_identity::HostIdentity::ThirdParty(id) => {
                record.manifest.id == *id && record.enabled
            }
        }) {
            record.status = ExtensionStatus::Error;
            record.last_error = Some("host_unavailable".to_string());
            record.contributions = ExtensionContributions::default();
        }
        Ok::<(), String>(())
    });
}
