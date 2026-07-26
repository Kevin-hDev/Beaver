use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolGroupEntry {
    pub id: &'static str,
    pub locked: bool,
    pub default_enabled: bool,
    pub tool_ids: &'static [&'static str],
}

const LOCKED_GROUPS: &[ToolGroupEntry] = &[
    group("terminal", true, true, &["bash"]),
    group(
        "files",
        true,
        true,
        &["read_file", "write_file", "edit_file", "list_dir"],
    ),
    group("file_search", true, true, &["grep", "glob"]),
    group("web", true, true, &["web_search", "web_fetch"]),
    group("mcp", true, true, &["search_mcp_tools"]),
];

const OPTIONAL_GROUPS: &[ToolGroupEntry] = &[
    group("skills", false, true, &["load_skill"]),
    group("user_choice", false, true, &["ask_user_choice"]),
    group(
        "subagents",
        false,
        true,
        super::tool_catalog::SUBAGENT_TOOLS,
    ),
    group("plan_mode", false, true, &["planmode", "exitplanmode"]),
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
    ),
    group("agent_diagnostics", false, false, &["agent_diagnostics"]),
    group(
        "git_branches",
        false,
        false,
        &["create_branch", "checkout_branch"],
    ),
    group(
        "forecast",
        false,
        false,
        &[
            "forecast_data_audit",
            "forecast",
            "forecast_models",
            "forecast_analyze",
            "forecast_read",
            "forecast_backtest",
            "forecast_compare_models",
        ],
    ),
    group(
        "spreadsheet",
        false,
        false,
        &["read_spreadsheet", "write_spreadsheet"],
    ),
    group(
        "document",
        false,
        false,
        &["read_document", "write_document"],
    ),
    group("images", false, false, &["read_image", "process_image"]),
];

const fn group(
    id: &'static str,
    locked: bool,
    default_enabled: bool,
    tool_ids: &'static [&'static str],
) -> ToolGroupEntry {
    ToolGroupEntry {
        id,
        locked,
        default_enabled,
        tool_ids,
    }
}

pub fn groups() -> Vec<ToolGroupEntry> {
    LOCKED_GROUPS
        .iter()
        .chain(OPTIONAL_GROUPS.iter())
        .copied()
        .collect()
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

        assert_eq!(plan_tools, ["planmode", "exitplanmode"]);
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

        assert_eq!(tools, super::super::tool_catalog::SUBAGENT_TOOLS);
    }

    #[test]
    fn locked_or_unknown_group_cannot_be_toggled() {
        assert!(optional_group_tool_ids("web").is_err());
        assert!(optional_group_tool_ids("unknown").is_err());
    }
}
