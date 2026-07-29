use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct PluginDescriptor {
    pub id: String,
    pub tool_count: usize,
    pub definition_count: usize,
}

pub struct SelectionPolicy<'a> {
    pub masked: bool,
    pub tool_capacity: usize,
    pub ordered_plugin_ids: &'a [String],
    pub capacity_plugin_ids: &'a [String],
    pub protected_plugin_ids: &'a [String],
    pub essential_plugin_ids: &'a [String],
    pub discovered_plugin_ids: &'a [String],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CapacityDecision {
    pub active_plugin_ids: Vec<String>,
    pub omitted_plugin_ids: Vec<String>,
}

pub fn decide_for_catalog(
    plugins: &[PluginDescriptor],
    catalog: &crate::services::extensions::CatalogSnapshot,
    masked: bool,
    tool_capacity: usize,
    discovered_plugin_ids: &[String],
) -> CapacityDecision {
    decide(
        plugins,
        SelectionPolicy {
            masked,
            tool_capacity,
            ordered_plugin_ids: &catalog.ordered_plugin_ids,
            capacity_plugin_ids: &catalog.capacity_plugin_ids,
            protected_plugin_ids: &catalog.protected_plugin_ids,
            essential_plugin_ids: &catalog.essential_plugin_ids,
            discovered_plugin_ids,
        },
    )
}

pub fn decide(plugins: &[PluginDescriptor], policy: SelectionPolicy<'_>) -> CapacityDecision {
    let known = plugins
        .iter()
        .map(|plugin| plugin.id.as_str())
        .collect::<HashSet<_>>();
    let capacity_overflow = plugins
        .iter()
        .map(|plugin| plugin.tool_count)
        .sum::<usize>()
        > policy.tool_capacity;
    let remaining_order = if capacity_overflow {
        policy.capacity_plugin_ids
    } else {
        policy.ordered_plugin_ids
    };
    let mut desired = Vec::with_capacity(known.len());
    let mut seen = HashSet::with_capacity(known.len());
    for id in policy
        .protected_plugin_ids
        .iter()
        .chain(policy.essential_plugin_ids)
        .chain(policy.discovered_plugin_ids)
        .chain((!policy.masked).then_some(remaining_order).into_iter().flatten())
    {
        if known.contains(id.as_str()) && seen.insert(id.as_str()) {
            desired.push(id.clone());
        }
    }
    let mut remaining = policy.tool_capacity;
    let mut decision = CapacityDecision::default();
    for id in desired {
        let Some(plugin) = plugins.iter().find(|plugin| plugin.id == id) else {
            continue;
        };
        if plugin.tool_count <= remaining {
            remaining = remaining.saturating_sub(plugin.tool_count);
            decision.active_plugin_ids.push(id);
        } else {
            decision.omitted_plugin_ids.push(id);
        }
    }
    decision
}

#[cfg(test)]
#[path = "extension_tool_selection_tests.rs"]
mod tests;
