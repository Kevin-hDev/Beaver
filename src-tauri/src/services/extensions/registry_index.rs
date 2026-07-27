use super::types::{ExtensionKind, ExtensionRecord, ExtensionTool};
use std::collections::HashSet;
use std::sync::{LazyLock, RwLock};

#[derive(Default)]
struct DynamicIndex {
    tools: Vec<ExtensionTool>,
    names: HashSet<String>,
    replacements: HashSet<String>,
}

static INDEX: LazyLock<RwLock<DynamicIndex>> =
    LazyLock::new(|| RwLock::new(DynamicIndex::default()));

pub fn rebuild(records: &[ExtensionRecord]) -> Result<(), String> {
    let tools: Vec<ExtensionTool> = records
        .iter()
        .filter(|record| record.kind == ExtensionKind::Local && record.enabled)
        .flat_map(|record| record.contributions.tools.iter().cloned())
        .collect();
    let names = tools.iter().map(|tool| tool.name.clone()).collect();
    let replacements = tools
        .iter()
        .filter(|tool| tool.replaces_core)
        .map(|tool| tool.name.clone())
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
        .map(|index| index.tools.clone())
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
            .find(|tool| tool.name == tool_name)
            .cloned()
    })
}
