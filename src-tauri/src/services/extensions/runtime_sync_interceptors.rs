use super::tool_interception::InterceptorRegistration;
use super::types::{ExtensionDiagnostic, ExtensionStatus, MAX_INTERCEPTORS};
use std::collections::{HashMap, HashSet};

pub(super) fn accepted(
    candidates: Vec<InterceptorRegistration>,
    diagnostics: &mut Vec<ExtensionDiagnostic>,
) -> Result<Vec<InterceptorRegistration>, String> {
    let active = super::registry::list()?
        .into_iter()
        .filter(|record| {
            record.enabled
                && record.status == ExtensionStatus::Active
                && !record.contributions.interceptors.is_empty()
        })
        .map(|record| record.manifest.id)
        .collect::<HashSet<_>>();
    let mut candidates = candidates
        .into_iter()
        .filter(|entry| active.contains(&entry.extension_id))
        .map(|entry| (entry.extension_id.clone(), entry))
        .collect::<HashMap<_, _>>();
    let mut accepted = Vec::new();
    for id in super::registry_catalog()?.ordered_plugin_ids {
        let Some(entry) = candidates.remove(&id) else {
            continue;
        };
        if accepted.len() < MAX_INTERCEPTORS {
            accepted.push(entry);
        } else {
            super::runtime_sync_apply::push_diagnostic(
                diagnostics,
                super::runtime_sync::runtime_diagnostic(
                    &id,
                    super::types::HOST_LOAD_STAGE_REGISTER,
                    super::types::DIAGNOSTIC_INTERCEPTION_BUDGET_EXHAUSTED,
                ),
            )?;
        }
    }
    Ok(accepted)
}
