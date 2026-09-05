use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

use super::discovery_limits::MAX_SELF_DECLARED_ESSENTIAL_PLUGINS;
use super::registry_index::IndexedPlugin;

#[derive(Clone, Default)]
pub struct CatalogSnapshot {
    pub text: String,
    pub version: String,
    pub ordered_plugin_ids: Vec<String>,
    pub capacity_plugin_ids: Vec<String>,
    pub protected_plugin_ids: Vec<String>,
    pub essential_plugin_ids: Vec<String>,
}

pub fn build(
    plugins: &[IndexedPlugin],
    protected_plugin_ids: &[String],
    scores: &BTreeMap<String, f64>,
) -> Result<CatalogSnapshot, String> {
    if plugins.len() > super::discovery_contract::HOST_MAX_EXTENSIONS {
        return Err(super::error_codes::LISTING_UNAVAILABLE.to_string());
    }
    let mut stable = plugins.to_vec();
    stable.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });
    let ids = stable
        .iter()
        .map(|plugin| plugin.id.as_str())
        .collect::<HashSet<_>>();
    let protected = protected_plugin_ids
        .iter()
        .filter(|id| ids.contains(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let protected_set = protected.iter().map(String::as_str).collect::<HashSet<_>>();
    let essential = stable
        .iter()
        .filter(|plugin| plugin.essential && !protected_set.contains(plugin.id.as_str()))
        .take(MAX_SELF_DECLARED_ESSENTIAL_PLUGINS)
        .map(|plugin| plugin.id.clone())
        .collect::<Vec<_>>();
    let preferred = protected
        .iter()
        .chain(&essential)
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let rest = stable
        .iter()
        .filter(|plugin| !preferred.contains(plugin.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let mut capacity_rest = rest.clone();
    capacity_rest.sort_by(|left, right| {
        score(scores, &right.id)
            .total_cmp(&score(scores, &left.id))
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.id.cmp(&right.id))
    });
    let by_id = stable
        .into_iter()
        .map(|plugin| (plugin.id.clone(), plugin))
        .collect::<BTreeMap<_, _>>();
    let ordered_plugin_ids = protected
        .iter()
        .chain(&essential)
        .cloned()
        .chain(rest.iter().map(|plugin| plugin.id.clone()))
        .collect::<Vec<_>>();
    let capacity_plugin_ids = protected
        .iter()
        .chain(&essential)
        .cloned()
        .chain(capacity_rest.iter().map(|plugin| plugin.id.clone()))
        .collect::<Vec<_>>();
    let ordered = ordered_plugin_ids
        .iter()
        .filter_map(|id| by_id.get(id).cloned())
        .collect::<Vec<_>>();
    let text = super::discovery_listing::compact_catalog(&ordered)?;
    let version = fingerprint(plugins, protected_plugin_ids);
    Ok(CatalogSnapshot {
        text,
        version,
        ordered_plugin_ids,
        capacity_plugin_ids,
        protected_plugin_ids: protected,
        essential_plugin_ids: essential,
    })
}

fn fingerprint(plugins: &[IndexedPlugin], protected_plugin_ids: &[String]) -> String {
    let mut stable = plugins.to_vec();
    stable.sort_by(|left, right| left.id.cmp(&right.id));
    let mut hash = Sha256::new();
    hash.update((stable.len() as u64).to_le_bytes());
    for plugin in &stable {
        hash_text(&mut hash, &plugin.id);
        hash_text(&mut hash, &plugin.name);
        hash_text(&mut hash, &plugin.version);
        hash_text(&mut hash, plugin.description.as_deref().unwrap_or_default());
        hash.update([plugin.essential as u8]);
        hash.update((plugin.tools.len() as u64).to_le_bytes());
        for tool in &plugin.tools {
            hash_text(&mut hash, &tool.name);
            hash_text(&mut hash, &tool.description);
            hash.update([tool.replaces_core as u8]);
            hash_text(&mut hash, &tool.parameters.to_string());
        }
        hash.update((plugin.skills.len() as u64).to_le_bytes());
        for skill in &plugin.skills {
            hash_text(&mut hash, &skill.id);
            hash_text(&mut hash, &skill.name);
            hash_text(&mut hash, &skill.description);
            hash_text(&mut hash, &skill.path);
        }
        hash.update((plugin.resources.len() as u64).to_le_bytes());
        for resource in &plugin.resources {
            hash_text(&mut hash, &resource.id);
            hash_text(&mut hash, &resource.name);
            hash_text(&mut hash, &resource.description);
            hash_text(&mut hash, &format!("{:?}", resource.resource_type));
            hash_text(&mut hash, &resource.path);
        }
    }
    hash.update((protected_plugin_ids.len() as u64).to_le_bytes());
    for id in protected_plugin_ids {
        hash_text(&mut hash, id);
    }
    format!("{:x}", hash.finalize())
}

fn hash_text(hash: &mut Sha256, value: &str) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value.as_bytes());
}

fn score(scores: &BTreeMap<String, f64>, id: &str) -> f64 {
    scores.get(id).copied().unwrap_or_default()
}

#[cfg(test)]
#[path = "discovery_catalog_tests.rs"]
mod tests;
