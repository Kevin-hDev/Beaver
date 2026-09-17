use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolGroupEntry {
    pub id: &'static str,
    pub locked: bool,
    pub default_enabled: bool,
    pub tool_ids: &'static [&'static str],
    #[serde(skip)]
    pub(crate) catalog_group: &'static str,
}

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

const LOCKED_GROUPS: &[ToolGroupEntry] = &[
    group("terminal", true, true, &["bash", "bash_control"], "core"),
    group(
        "files",
        true,
        true,
        &["read_file", "write_file", "edit_file", "list_dir"],
        "core",
    ),
    group("file_search", true, true, &["grep", "glob"], "core"),
    group("web", true, true, &["web_search", "web_fetch"], "web"),
    group("mcp", true, true, &["search_mcp_tools"], "mcp"),
];

const CATALOG_ONLY_GROUPS: &[ToolGroupEntry] = &[group(
    "extensions",
    true,
    true,
    &[
        crate::services::extensions::LIST_EXTENSIONS_TOOL_NAME,
        crate::services::extensions::INSPECT_EXTENSIONS_TOOL_NAME,
        super::tool_extension_resource::NAME,
    ],
    "extensions",
)];

const OPTIONAL_GROUPS: &[ToolGroupEntry] = &[
    group("skills", false, true, &["load_skill"], "workflow"),
    group(
        "automations",
        false,
        true,
        &["manage_automation"],
        "automation",
    ),
    group(
        "user_choice",
        false,
        true,
        &["ask_user_choice"],
        "workflow",
    ),
    group("subagents", false, true, SUBAGENT_TOOLS, "subagents"),
    group("plan_mode", false, true, &["plan_mode"], "workflow"),
    group(
        "todo_list",
        false,
        false,
        &[
            "todo_write",
            "todo_history",
            "todo_pause",
            "todo_resume",
            "todo_delete",
        ],
        "todo",
    ),
    group(
        "git_branches",
        false,
        false,
        &["create_branch", "checkout_branch"],
        "git",
    ),
    group(
        "forecast",
        false,
        false,
        &[
            "forecast_data_audit",
            "forecast_run",
            "forecast_models",
            "forecast_analyze",
            "forecast_read",
            "forecast_backtest",
            "forecast_compare_models",
        ],
        "forecast",
    ),
    group(
        "spreadsheet",
        false,
        false,
        &["read_spreadsheet", "write_spreadsheet"],
        "office",
    ),
    group(
        "document",
        false,
        false,
        &["read_document", "write_document"],
        "office",
    ),
    group("images", false, false, &["transform_image"], "office"),
];

const fn group(
    id: &'static str,
    locked: bool,
    default_enabled: bool,
    tool_ids: &'static [&'static str],
    catalog_group: &'static str,
) -> ToolGroupEntry {
    ToolGroupEntry {
        id,
        locked,
        default_enabled,
        tool_ids,
        catalog_group,
    }
}

pub fn groups() -> Vec<ToolGroupEntry> {
    LOCKED_GROUPS
        .iter()
        .chain(OPTIONAL_GROUPS.iter())
        .copied()
        .collect()
}

pub(crate) fn catalog_groups() -> impl Iterator<Item = &'static ToolGroupEntry> {
    LOCKED_GROUPS
        .iter()
        .chain(CATALOG_ONLY_GROUPS.iter())
        .chain(OPTIONAL_GROUPS.iter())
}

pub fn optional_group_tool_ids(group_id: &str) -> Result<&'static [&'static str], String> {
    if LOCKED_GROUPS.iter().any(|group| group.id == group_id) {
        return Err("Ce groupe d'outils est verrouillé.".into());
    }
    OPTIONAL_GROUPS
        .iter()
        .find(|group| group.id == group_id)
        .map(|group| group.tool_ids)
        .ok_or_else(|| "Groupe d'outils inconnu.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_tools_are_grouped_and_locked() {
        let web = groups()
            .into_iter()
            .find(|group| group.id == "web")
            .expect("web group");

        assert!(web.locked);
        assert_eq!(web.tool_ids, ["web_search", "web_fetch"]);
    }

    #[test]
    fn optional_group_returns_all_real_tool_ids() {
        let plan_tools = optional_group_tool_ids("plan_mode").unwrap();

        assert_eq!(plan_tools, ["plan_mode"]);
    }

    #[test]
    fn forecast_group_contains_the_data_audit() {
        let tools = optional_group_tool_ids("forecast").unwrap();

        assert!(tools.contains(&"forecast_data_audit"));
        assert!(tools.contains(&"forecast_backtest"));
        assert!(tools.contains(&"forecast_compare_models"));
    }

    #[test]
    fn subagent_group_contains_all_control_tools() {
        let tools = optional_group_tool_ids("subagents").unwrap();

        assert_eq!(tools, SUBAGENT_TOOLS);
    }

    #[test]
    fn internal_office_tools_remain_in_the_tools_catalog() {
        assert_eq!(
            optional_group_tool_ids("spreadsheet").unwrap(),
            ["read_spreadsheet", "write_spreadsheet"]
        );
        assert_eq!(
            optional_group_tool_ids("document").unwrap(),
            ["read_document", "write_document"]
        );
        assert_eq!(
            optional_group_tool_ids("images").unwrap(),
            ["transform_image"]
        );
    }

    #[test]
    fn locked_or_unknown_group_cannot_be_toggled() {
        assert!(optional_group_tool_ids("web").is_err());
        assert!(optional_group_tool_ids("unknown").is_err());
    }
}
