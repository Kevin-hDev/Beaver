use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolCatalogEntry {
    pub id: &'static str,
    pub locked: bool,
    pub default_enabled: bool,
    pub group: &'static str,
}

pub const MAX_OPTIONAL_TOOLS: usize = 34;
pub const SUBAGENT_TOOLS: &[&str] = &[
    "delegate_task",
    "list_subagents",
    "get_subagent",
    "cancel_subagent",
    "message_subagent",
    "archive_subagent",
    "inspect_subagent_changes",
    "apply_subagent_changes",
    "discard_subagent_changes",
];

const LOCKED_TOOLS: &[ToolCatalogEntry] = &[
    locked("bash", "core"),
    locked("read_file", "core"),
    locked("write_file", "core"),
    locked("edit_file", "core"),
    locked("list_dir", "core"),
    locked("grep", "core"),
    locked("glob", "core"),
    locked("web_search", "web"),
    locked("web_fetch", "web"),
    locked("search_mcp_tools", "mcp"),
];

const OPTIONAL_TOOLS: &[ToolCatalogEntry] = &[
    optional_default("load_skill", "workflow"),
    optional_default("ask_user_choice", "workflow"),
    optional_default("delegate_task", "subagents"),
    optional_default("list_subagents", "subagents"),
    optional_default("get_subagent", "subagents"),
    optional_default("cancel_subagent", "subagents"),
    optional_default("message_subagent", "subagents"),
    optional_default("archive_subagent", "subagents"),
    optional_default("inspect_subagent_changes", "subagents"),
    optional_default("apply_subagent_changes", "subagents"),
    optional_default("discard_subagent_changes", "subagents"),
    optional_default("planmode", "workflow"),
    optional_default("exitplanmode", "workflow"),
    optional_off("todo_write", "todo"),
    optional_off("todo_history", "todo"),
    optional_off("todo_pause", "todo"),
    optional_off("todo_resume", "todo"),
    optional_off("todo_delete", "todo"),
    optional_off("agent_diagnostics", "todo"),
    optional_off("create_branch", "git"),
    optional_off("checkout_branch", "git"),
    optional_off("forecast_data_audit", "forecast"),
    optional_off("forecast", "forecast"),
    optional_off("forecast_models", "forecast"),
    optional_off("forecast_analyze", "forecast"),
    optional_off("forecast_read", "forecast"),
    optional_off("forecast_backtest", "forecast"),
    optional_off("forecast_compare_models", "forecast"),
    optional_off("read_spreadsheet", "office"),
    optional_off("write_spreadsheet", "office"),
    optional_off("read_document", "office"),
    optional_off("write_document", "office"),
    optional_off("read_image", "office"),
    optional_off("process_image", "office"),
];

const fn locked(id: &'static str, group: &'static str) -> ToolCatalogEntry {
    ToolCatalogEntry {
        id,
        locked: true,
        default_enabled: true,
        group,
    }
}

const fn optional_default(id: &'static str, group: &'static str) -> ToolCatalogEntry {
    ToolCatalogEntry {
        id,
        locked: false,
        default_enabled: true,
        group,
    }
}

const fn optional_off(id: &'static str, group: &'static str) -> ToolCatalogEntry {
    ToolCatalogEntry {
        id,
        locked: false,
        default_enabled: false,
        group,
    }
}

pub fn catalog() -> Vec<ToolCatalogEntry> {
    LOCKED_TOOLS
        .iter()
        .chain(OPTIONAL_TOOLS.iter())
        .copied()
        .collect()
}

pub fn default_enabled_optional_tools() -> Vec<String> {
    OPTIONAL_TOOLS
        .iter()
        .filter(|tool| tool.default_enabled)
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
    OPTIONAL_TOOLS
        .iter()
        .filter(|tool| selected.contains(tool.id))
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
    LOCKED_TOOLS.iter().any(|tool| tool.id == tool_id)
}

pub fn is_optional_tool(tool_id: &str) -> bool {
    OPTIONAL_TOOLS.iter().any(|tool| tool.id == tool_id)
}

pub fn is_subagent_tool(tool_id: &str) -> bool {
    SUBAGENT_TOOLS.contains(&tool_id)
}

pub fn is_enabled(tool_id: &str, enabled_optional_tools: &[String]) -> bool {
    is_locked_tool(tool_id)
        || (is_optional_tool(tool_id)
            && enabled_optional_tools
                .iter()
                .any(|enabled| enabled == tool_id))
}

pub fn filter_tool_definitions(defs: Vec<Value>, enabled_optional_tools: &[String]) -> Vec<Value> {
    defs.into_iter()
        .filter(|def| {
            tool_name(def)
                .as_deref()
                .is_some_and(|name| is_enabled(name, enabled_optional_tools))
        })
        .collect()
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
    has_tool(names, "planmode") && has_tool(names, "exitplanmode")
}

fn tool_name(def: &Value) -> Option<String> {
    def.get("function")?
        .get("name")?
        .as_str()
        .map(ToString::to_string)
}
