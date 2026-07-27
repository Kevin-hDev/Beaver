use super::protocol::{HostExtensionSpec, SyncResult};
use super::types::{ExtensionApiLevel, MAX_EXTENSIONS};
use std::collections::HashSet;

pub fn build_specs(
    records: Vec<super::types::ExtensionRecord>,
) -> Result<Vec<HostExtensionSpec>, String> {
    let mut specs = Vec::with_capacity(records.len());
    for record in records.into_iter().take(MAX_EXTENSIONS) {
        super::registry::mark_loading(&record.manifest.id)?;
        let main = match super::manifest::resolve_record_entry(&record) {
            Ok(main) => main,
            Err(_) => {
                super::registry::mark_error(&record.manifest.id);
                continue;
            }
        };
        let Some(main_path) = main.to_str() else {
            super::registry::mark_error(&record.manifest.id);
            continue;
        };
        specs.push(HostExtensionSpec {
            id: record.manifest.id.clone(),
            main_path: main_path.to_string(),
            manifest: record.manifest,
        });
    }
    Ok(specs)
}

pub fn apply(response: SyncResult, specs: &[HostExtensionSpec]) -> Result<usize, String> {
    let requested: HashSet<String> = specs.iter().map(|spec| spec.id.clone()).collect();
    let mut received = HashSet::new();
    let mut active = 0;
    for loaded in response.extensions.into_iter().take(MAX_EXTENSIONS) {
        if !requested.contains(&loaded.id) || !received.insert(loaded.id.clone()) {
            return Err("Réponse de l'hôte d'extensions invalide.".to_string());
        }
        let Some(contributions) = loaded.contributions.filter(|_| loaded.error.is_none()) else {
            super::registry::mark_error(&loaded.id);
            continue;
        };
        let advanced = specs.iter().any(|spec| {
            spec.id == loaded.id && spec.manifest.api_level == ExtensionApiLevel::Advanced
        });
        if !advanced && contributions.tools.iter().any(|tool| tool.replaces_core) {
            super::registry::mark_error(&loaded.id);
            continue;
        }
        if super::registry::apply_loaded(&loaded.id, contributions).is_err() {
            super::registry::mark_error(&loaded.id);
        } else {
            active += 1;
        }
    }
    for missing in requested.difference(&received) {
        super::registry::mark_error(missing);
    }
    Ok(active)
}
