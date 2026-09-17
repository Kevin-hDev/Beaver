use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

pub use super::tool_catalog_filter::filter_tool_definitions;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolCatalogEntry {
    pub id: &'static str,
    pub locked: bool,
    pub default_enabled: bool,
    pub group: &'static str,
}

pub const MAX_OPTIONAL_TOOLS: usize = 32;
pub use super::tool_group_catalog::SUBAGENT_TOOLS;

fn entries() -> impl Iterator<Item = ToolCatalogEntry> {
    super::tool_group_catalog::catalog_groups().flat_map(|group| {
        group
            .tool_ids
            .iter()
            .copied()
            .map(|id| ToolCatalogEntry {
                id,
                locked: group.locked,
                default_enabled: group.default_enabled,
                group: group.catalog_group,
            })
    })
}

pub fn catalog() -> Vec<ToolCatalogEntry> {
    entries().collect()
}

pub fn default_enabled_optional_tools() -> Vec<String> {
    entries()
        .filter(|tool| !tool.locked && tool.default_enabled)
        .map(|tool| tool.id.to_string())
        .collect()
}

pub fn normalize_enabled_optional_tools(input: &[String]) -> Vec<String> {
    let mut selected: BTreeSet<&str> = input.iter().map(String::as_str).collect();
    if selected.contains("delegate_task") {
        selected.extend(SUBAGENT_TOOLS.iter().copied());
    } else {
        selected.retain(|tool_id| !SUBAGENT_TOOLS.contains(tool_id));
    }
    entries()
        .filter(|tool| selected.contains(tool.id))
        .filter(|tool| !tool.locked)
        .take(MAX_OPTIONAL_TOOLS)
        .map(|tool| tool.id.to_string())
        .collect()
}

pub fn validate_optional_tool_id(tool_id: &str) -> Result<(), String> {
    if is_locked_tool(tool_id) {
        return Err("Ce tool est verrouillé.".to_string());
    }
    if !is_optional_tool(tool_id) {
        return Err("Tool inconnu.".to_string());
    }
    Ok(())
}

pub fn is_locked_tool(tool_id: &str) -> bool {
    entries().any(|tool| tool.locked && tool.id == tool_id)
}

pub fn is_optional_tool(tool_id: &str) -> bool {
    entries().any(|tool| !tool.locked && tool.id == tool_id)
}

pub fn is_enabled(tool_id: &str, enabled_optional_tools: &[String]) -> bool {
    is_locked_tool(tool_id)
        || (is_optional_tool(tool_id)
            && enabled_optional_tools
                .iter()
                .any(|enabled| enabled == tool_id))
}

pub fn tool_names(defs: &[Value]) -> Vec<String> {
    defs.iter().filter_map(tool_name).collect()
}

pub fn has_tool(names: &[String], tool_id: &str) -> bool {
    names.iter().any(|name| name == tool_id)
}

pub fn has_any_tool(names: &[String], tool_ids: &[&str]) -> bool {
    tool_ids.iter().any(|tool_id| has_tool(names, tool_id))
}

pub fn has_plan_tools(names: &[String]) -> bool {
    has_tool(names, "plan_mode")
}

pub(super) fn tool_name(def: &Value) -> Option<String> {
    def.get("function")?
        .get("name")?
        .as_str()
        .map(ToString::to_string)
}
