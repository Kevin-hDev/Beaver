use super::types::{
    ExtensionContributions, ExtensionKind, ExtensionRecord, ExtensionStatus, MAX_TOOLS,
};
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
    let mut active = 0;
    super::registry::mutate(|records| {
        apply_loaded_results(records, enabled_ids, &successful, failures, &mut active);
        Ok::<(), String>(())
    })?;
    Ok(active)
}

fn apply_loaded_results(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::extensions::types::{
        ExtensionApiLevel, ExtensionEffect, ExtensionManifest, ExtensionTool,
    };
    use serde_json::json;

    fn contributions(name: &str) -> ExtensionContributions {
        ExtensionContributions {
            tools: vec![ExtensionTool {
                name: name.to_string(),
                description: "Tool".to_string(),
                parameters: json!({"type": "object"}),
                effect: ExtensionEffect::ReadOnly,
                replaces_core: false,
            }],
            events: Vec::new(),
        }
    }

    fn record(id: &str) -> ExtensionRecord {
        ExtensionRecord {
            manifest: ExtensionManifest {
                id: id.to_string(),
                name: id.to_string(),
                version: "1.0.0".to_string(),
                beaver_api: "1".to_string(),
                runtime: "node".to_string(),
                main: Some("index.mjs".to_string()),
                ui: None,
                access: "full".to_string(),
                api_level: ExtensionApiLevel::Stable,
                essential: false,
                author: None,
                homepage: None,
                description: None,
            },
            kind: ExtensionKind::Local,
            source: "test".to_string(),
            origin: None,
            enabled: true,
            trusted: true,
            show_in_chat: true,
            status: ExtensionStatus::Loading,
            last_error: None,
            last_activated_at: None,
            sensitive_access_granted: false,
            contributions: ExtensionContributions::default(),
        }
    }

    #[test]
    fn second_plugin_with_the_same_canonical_tool_name_is_rejected() {
        let successful = HashMap::from([
            ("plugin-a".to_string(), contributions("shared.tool")),
            ("plugin-b".to_string(), contributions("shared.tool")),
        ]);
        let enabled = HashSet::from(["plugin-a".to_string(), "plugin-b".to_string()]);
        let mut records = vec![record("plugin-a"), record("plugin-b")];
        let mut active = 0;

        super::apply_loaded_results(
            &mut records,
            &enabled,
            &successful,
            &BTreeMap::new(),
            &mut active,
        );

        assert_eq!(active, 1);
        assert_eq!(records[0].status, ExtensionStatus::Active);
        assert_eq!(records[1].status, ExtensionStatus::Error);
        assert_eq!(records[1].last_error.as_deref(), Some("load_failed"));
        assert!(records[1].contributions.tools.is_empty());
    }
}
