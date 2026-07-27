use super::types::{ExtensionContributions, ExtensionKind, ExtensionStatus, MAX_TOOLS};
use std::collections::{HashMap, HashSet};

pub fn mark_enabled_loading() -> Result<(), String> {
    super::registry::mutate(|records| {
        for record in records
            .iter_mut()
            .filter(|record| record.kind != ExtensionKind::External && record.enabled)
        {
            record.status = ExtensionStatus::Loading;
            record.last_error = None;
        }
        Ok(())
    })
}

pub fn apply_results(
    enabled_ids: &HashSet<String>,
    successful: HashMap<String, ExtensionContributions>,
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
                record.last_error = Some("load_failed".to_string());
            }
        }
        Ok(())
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
        Ok(())
    });
}
