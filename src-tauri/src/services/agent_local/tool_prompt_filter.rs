const TODO_SECTION: &str = "# Todo list";
const INTERACTIVE_SECTION: &str = "# Interactive choices";

pub fn filter_system_prompt(content: &str, enabled_tool_names: &[String]) -> String {
    let mut lines = Vec::new();
    let mut skip_section = false;
    let has_todo = super::tool_catalog::has_any_tool(
        enabled_tool_names,
        &[
            "todo_write",
            "todo_history",
            "todo_pause",
            "todo_resume",
            "todo_delete",
            "agent_diagnostics",
        ],
    );
    let has_interactive = super::tool_catalog::has_tool(enabled_tool_names, "ask_user_choice");

    for line in content.lines() {
        if line.starts_with("# ") {
            skip_section = (!has_todo && line == TODO_SECTION)
                || (!has_interactive && line == INTERACTIVE_SECTION);
        }
        if skip_section || should_drop_line(line, enabled_tool_names) {
            continue;
        }
        lines.push(line);
    }

    collapse_blank_lines(&lines.join("\n"))
}

fn should_drop_line(line: &str, enabled_tool_names: &[String]) -> bool {
    for entry in super::tool_catalog::catalog() {
        if !super::tool_catalog::has_tool(enabled_tool_names, entry.id)
            && mentions_tool(line, entry.id)
        {
            return true;
        }
    }

    let lower = line.to_lowercase();
    if !super::tool_catalog::has_tool(enabled_tool_names, "delegate_task")
        && lower.contains("subagent")
    {
        return true;
    }
    if !super::tool_catalog::has_tool(enabled_tool_names, "write_spreadsheet")
        && lower.contains("spreadsheet")
        && lower.contains("formula")
    {
        return true;
    }
    false
}

fn mentions_tool(line: &str, tool_id: &str) -> bool {
    line.contains(&format!("**{tool_id}**"))
        || line.contains(&format!("`{tool_id}`"))
        || line.contains(tool_id)
}

fn collapse_blank_lines(input: &str) -> String {
    let mut out = Vec::new();
    let mut blank_count = 0;
    for line in input.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                out.push(line);
            }
        } else {
            blank_count = 0;
            out.push(line);
        }
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_disabled_tool_mentions_and_sections() {
        let enabled = vec![
            "bash".to_string(),
            "read_file".to_string(),
            "load_skill".to_string(),
        ];
        let filtered = filter_system_prompt(
            "# Capabilities\n- **bash**: Run.\n- **todo_write**: Track.\n- **load_skill**: Load.\n\n# Todo list\nUse todo_write.\n\n# Interactive choices\nUse ask_user_choice.",
            &enabled,
        );

        assert!(filtered.contains("**bash**"));
        assert!(filtered.contains("**load_skill**"));
        assert!(!filtered.contains("todo_write"));
        assert!(!filtered.contains("ask_user_choice"));
        assert!(!filtered.contains("# Todo list"));
    }
}
