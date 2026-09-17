use super::discovery_catalog::CatalogSnapshot;
use super::types::{ExtensionRecord, ExtensionResource, ExtensionSkill, ExtensionTool};
use std::collections::{BTreeMap, HashSet};
use std::sync::{LazyLock, RwLock};

#[derive(Clone)]
pub(crate) struct IndexedPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub essential: bool,
    pub tools: Vec<ExtensionTool>,
    pub skills: Vec<ExtensionSkill>,
    pub resources: Vec<ExtensionResource>,
}

#[derive(Clone)]
pub(crate) struct IndexedTool {
    pub extension_id: String,
    pub extension_name: String,
    pub tool: ExtensionTool,
}

#[derive(Default)]
struct DynamicIndex {
    available: bool,
    unavailable_reason: Option<&'static str>,
    plugins: Vec<IndexedPlugin>,
    tools: Vec<IndexedTool>,
    names: HashSet<String>,
    replacements: HashSet<String>,
    catalog: CatalogSnapshot,
}

static INDEX: LazyLock<RwLock<DynamicIndex>> =
    LazyLock::new(|| RwLock::new(DynamicIndex::default()));

pub fn rebuild(records: &[ExtensionRecord]) -> Result<(), String> {
    let preferences = super::discovery_preferences::sanitize(records)?;
    let plugins = plugins_from_records(records);
    let tools = plugins
        .iter()
        .flat_map(|plugin| {
            plugin.tools.iter().cloned().map(|tool| IndexedTool {
                extension_id: plugin.id.clone(),
                extension_name: plugin.name.clone(),
                tool,
            })
        })
        .collect::<Vec<_>>();
    let names = tools
        .iter()
        .map(|indexed| indexed.tool.name.clone())
        .collect();
    let replacements = tools
        .iter()
        .filter(|indexed| indexed.tool.replaces_core)
        .map(|indexed| indexed.tool.name.clone())
        .collect();
    let scores = usage_scores();
    let next_catalog =
        super::discovery_catalog::build(&plugins, &preferences.protected_plugin_ids, &scores)?;
    let previous_catalog = INDEX
        .read()
        .map(|index| index.catalog.clone())
        .unwrap_or_default();
    let catalog = stable_catalog(previous_catalog, next_catalog);
    let mut index = INDEX
        .write()
        .map_err(|_| super::error_codes::REGISTRY_UNAVAILABLE.to_string())?;
    *index = DynamicIndex {
        available: true,
        unavailable_reason: None,
        plugins,
        tools,
        names,
        replacements,
        catalog,
    };
    Ok(())
}

pub(super) fn plugins_from_records(records: &[ExtensionRecord]) -> Vec<IndexedPlugin> {
    records
        .iter()
        .filter(|record| record.enabled && record.trusted)
        .map(|record| IndexedPlugin {
            id: record.manifest.id.clone(),
            name: record.manifest.name.clone(),
            version: record.manifest.version.clone(),
            description: record.manifest.description.clone(),
            essential: record.manifest.essential,
            tools: record.contributions.tools.clone(),
            skills: record.contributions.skills.clone(),
            resources: record.contributions.resources.clone(),
        })
        .collect()
}

fn usage_scores() -> BTreeMap<String, f64> {
    usage_scores_with(super::discovery_usage::scores)
}

fn usage_scores_with(
    load: impl FnOnce() -> Result<BTreeMap<String, f64>, String>,
) -> BTreeMap<String, f64> {
    load().unwrap_or_else(|_| {
        ::log::warn!("[extensions] usage scores unavailable");
        BTreeMap::new()
    })
}

fn stable_catalog(previous: CatalogSnapshot, next: CatalogSnapshot) -> CatalogSnapshot {
    if previous.version == next.version {
        previous
    } else {
        next
    }
}

pub fn dynamic_tools() -> Vec<ExtensionTool> {
    INDEX
        .read()
        .map(|index| {
            index
                .tools
                .iter()
                .map(|indexed| indexed.tool.clone())
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn indexed_plugins() -> Vec<IndexedPlugin> {
    INDEX
        .read()
        .map(|index| index.plugins.clone())
        .unwrap_or_default()
}

pub(crate) fn indexed_plugins_with_catalog_version() -> Result<(Vec<IndexedPlugin>, String), ()> {
    INDEX
        .read()
        .map(|index| (index.plugins.clone(), index.catalog.version.clone()))
        .map_err(|_| ())
}

pub(crate) fn catalog_snapshot() -> CatalogSnapshot {
    INDEX
        .read()
        .map(|index| index.catalog.clone())
        .unwrap_or_default()
}

pub(crate) fn plugin_id_for_tool(tool_name: &str) -> Option<String> {
    INDEX.read().ok().and_then(|index| {
        index
            .tools
            .iter()
            .find(|indexed| indexed.tool.name == tool_name)
            .map(|indexed| indexed.extension_id.clone())
    })
}

pub(crate) fn dynamic_tool_names() -> Vec<String> {
    INDEX
        .read()
        .map(|index| index.names.iter().cloned().collect())
        .unwrap_or_default()
}

pub fn is_dynamic_tool(tool_name: &str) -> bool {
    INDEX
        .read()
        .map(|index| index.names.contains(tool_name))
        .unwrap_or(false)
}

pub fn is_replacement(tool_name: &str) -> bool {
    INDEX
        .read()
        .map(|index| index.replacements.contains(tool_name))
        .unwrap_or(false)
}

pub fn dynamic_tool(tool_name: &str) -> Option<ExtensionTool> {
    INDEX.read().ok().and_then(|index| {
        index
            .tools
            .iter()
            .find(|indexed| indexed.tool.name == tool_name)
            .map(|indexed| indexed.tool.clone())
    })
}

pub(crate) fn indexed_tool(tool_name: &str) -> Option<IndexedTool> {
    INDEX.read().ok().and_then(|index| {
        index
            .tools
            .iter()
            .find(|indexed| indexed.tool.name == tool_name)
            .cloned()
    })
}

#[cfg(test)]
#[path = "registry_index_tests.rs"]
mod tests;

#[path = "registry_availability.rs"]
mod availability;
pub(super) use availability::mark_unavailable;
pub(crate) use availability::{registry_availability, registry_catalog};
