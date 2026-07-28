use super::types::{ExtensionKind, ExtensionRecord, ExtensionTool};
use std::collections::HashSet;
use std::sync::{LazyLock, RwLock};

#[derive(Clone)]
pub(crate) struct IndexedTool {
    pub extension_id: String,
    pub extension_name: String,
    pub extension_description: String,
    pub tool: ExtensionTool,
}

#[derive(Default)]
struct DynamicIndex {
    tools: Vec<IndexedTool>,
    names: HashSet<String>,
    replacements: HashSet<String>,
}

static INDEX: LazyLock<RwLock<DynamicIndex>> =
    LazyLock::new(|| RwLock::new(DynamicIndex::default()));

pub fn rebuild(records: &[ExtensionRecord]) -> Result<(), String> {
    let tools: Vec<IndexedTool> = records
        .iter()
        .filter(|record| record.kind != ExtensionKind::External && record.enabled)
        .flat_map(|record| {
            record
                .contributions
                .tools
                .iter()
                .cloned()
                .map(|tool| IndexedTool {
                    extension_id: record.manifest.id.clone(),
                    extension_name: record.manifest.name.clone(),
                    extension_description: record.manifest.description.clone().unwrap_or_default(),
                    tool,
                })
        })
        .collect();
    let names = tools
        .iter()
        .map(|indexed| indexed.tool.name.clone())
        .collect();
    let replacements = tools
        .iter()
        .filter(|indexed| indexed.tool.replaces_core)
        .map(|indexed| indexed.tool.name.clone())
        .collect();
    let mut index = INDEX
        .write()
        .map_err(|_| "Index d'extensions indisponible.".to_string())?;
    *index = DynamicIndex {
        tools,
        names,
        replacements,
    };
    Ok(())
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

pub(crate) fn indexed_tools() -> Vec<IndexedTool> {
    INDEX
        .read()
        .map(|index| index.tools.clone())
        .unwrap_or_default()
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
