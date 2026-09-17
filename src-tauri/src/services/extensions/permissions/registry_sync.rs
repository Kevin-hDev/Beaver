use super::types::{ExtensionContributions, ExtensionRecord, ExtensionStatus, MAX_TOOLS};
use std::collections::{BTreeMap, HashMap, HashSet};

pub fn mark_loading(eligible_ids: &HashSet<String>) -> Result<(), String> {
    super::registry::mutate(|records| {
        mark_loading_records(records, eligible_ids);
        Ok::<(), String>(())
    })
}

pub(super) fn mark_loading_records(
    records: &mut [ExtensionRecord],
    eligible_ids: &HashSet<String>,
) {
    for record in records
        .iter_mut()
        .filter(|record| eligible_ids.contains(&record.manifest.id))
    {
        record.status = ExtensionStatus::Loading;
        record.last_error = None;
    }
}

pub fn apply_results(
    enabled_ids: &HashSet<String>,
    successful: HashMap<String, ExtensionContributions>,
    failures: &BTreeMap<String, String>,
) -> Result<usize, String> {
    let total_tools = successful
        .values()
        .try_fold(0_usize, |total, contributions| {
            super::validation::contributions(contributions)?;
            Ok::<usize, String>(total.saturating_add(contributions.tools.len()))
        })?;
    if total_tools > MAX_TOOLS {
        return Err(super::error_codes::LIMIT_REACHED.to_string());
    }
    let mut active = 0;
    super::registry::mutate(|records| {
        apply_loaded_results(records, enabled_ids, &successful, failures, &mut active);
        Ok::<(), String>(())
    })?;
    Ok(active)
}

pub(super) fn apply_loaded_results(
    records: &mut [ExtensionRecord],
    enabled_ids: &HashSet<String>,
    successful: &HashMap<String, ExtensionContributions>,
    failures: &BTreeMap<String, String>,
    active: &mut usize,
) {
    let accepted = accept_unique_tools(
        records.iter().map(|record| record.manifest.id.as_str()),
        successful,
    );
    for record in records
        .iter_mut()
        .filter(|record| enabled_ids.contains(&record.manifest.id))
    {
        if let Some(contributions) = successful
            .get(&record.manifest.id)
            .filter(|_| accepted.contains(&record.manifest.id))
        {
            record.contributions = contributions.clone();
            record.status = ExtensionStatus::Active;
            record.last_error = None;
            *active += 1;
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
}

fn accept_unique_tools<'a>(
    ordered_ids: impl IntoIterator<Item = &'a str>,
    successful: &HashMap<String, ExtensionContributions>,
) -> HashSet<String> {
    let mut names = HashSet::new();
    let mut accepted = HashSet::new();
    for id in ordered_ids {
        let Some(contributions) = successful.get(id) else {
            continue;
        };
        if contributions
            .tools
            .iter()
            .any(|tool| names.contains(&tool.name))
        {
            continue;
        }
        names.extend(contributions.tools.iter().map(|tool| tool.name.clone()));
        accepted.insert(id.to_string());
    }
    accepted
}

pub fn mark_all_enabled_error() {
    let _ = super::registry::mutate(|records| {
        apply_all_enabled_error(records);
        Ok::<(), String>(())
    });
}

pub(super) fn apply_all_enabled_error(records: &mut [ExtensionRecord]) {
    for record in records.iter_mut().filter(|record| record.enabled) {
        record.status = ExtensionStatus::Error;
        record.last_error = Some(super::error_codes::HOST_UNAVAILABLE.to_string());
    }
}

pub(super) use super::registry_failure::{mark_identity_error, mark_identity_stop_unconfirmed};
