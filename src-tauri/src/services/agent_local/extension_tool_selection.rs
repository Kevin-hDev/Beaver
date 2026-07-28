use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDescriptor {
    pub id: String,
    pub tool_count: usize,
    pub replaces_core: bool,
}

pub struct SelectionPolicy<'a> {
    pub masked: bool,
    pub tool_capacity: usize,
    pub ordered_plugin_ids: &'a [String],
    pub protected_plugin_ids: &'a [String],
    pub essential_plugin_ids: &'a [String],
    pub discovered_plugin_ids: &'a [String],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CapacityDecision {
    pub active_plugin_ids: Vec<String>,
    pub omitted_plugin_ids: Vec<String>,
}

pub fn decide(plugins: &[PluginDescriptor], policy: SelectionPolicy<'_>) -> CapacityDecision {
    let known = plugins.iter().map(|plugin| plugin.id.as_str()).collect::<HashSet<_>>();
    let replacements = policy
        .ordered_plugin_ids
        .iter()
        .filter(|id| {
            plugins
                .iter()
                .any(|plugin| &plugin.id == *id && plugin.replaces_core)
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut desired = Vec::with_capacity(known.len());
    let mut seen = HashSet::with_capacity(known.len());
    for id in replacements
        .iter()
        .chain(policy.protected_plugin_ids)
        .chain(policy.essential_plugin_ids)
        .chain(policy.discovered_plugin_ids)
        .chain((!policy.masked).then_some(policy.ordered_plugin_ids).into_iter().flatten())
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
