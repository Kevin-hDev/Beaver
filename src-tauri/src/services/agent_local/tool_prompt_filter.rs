/// Sections removed as a whole when their tool group is disabled. Line-level filtering is not
/// enough for these: their bullets do not all name a tool, so removing them one by one would
/// leave orphans behind.
const INTERACTIVE_SECTION: &str = "# Interactive choices";
const SUBAGENT_SECTION: &str = "# Working with subagents";

pub fn filter_system_prompt(content: &str, enabled_tool_names: &[String]) -> String {
    let mut lines = Vec::new();
    let mut skip_section = false;
    let has_interactive = super::tool_catalog::has_tool(enabled_tool_names, "ask_user_choice");
    let has_subagents = super::tool_catalog::has_tool(enabled_tool_names, "delegate_task");

    for line in content.lines() {
        if line.starts_with("# ") {
            skip_section = (!has_interactive && line == INTERACTIVE_SECTION)
                || (!has_subagents && line == SUBAGENT_SECTION);
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
            "# Capabilities\n- **bash**: Run.\n- **todo_write**: Track.\n- **load_skill**: Load.\n\n# Interactive choices\nUse ask_user_choice.",
            &enabled,
        );

        assert!(filtered.contains("**bash**"));
        assert!(filtered.contains("**load_skill**"));
        assert!(!filtered.contains("todo_write"));
        assert!(!filtered.contains("ask_user_choice"));
        assert!(!filtered.contains("# Interactive choices"));
    }

    /// The subagent section has bullets that name no tool at all. Line-level filtering would
    /// keep those and leave a section body without its heading.
    #[test]
    fn subagent_section_leaves_no_orphan_bullet_when_delegation_is_off() {
        let filtered = filter_system_prompt(
            &format!(
                "# Rules\nStay careful.\n\n{}\n\n# Style\nBe brief.",
                super::super::subagent_parent_guidance::PARENT_GUIDANCE
            ),
            &["bash".to_string()],
        );

        assert!(filtered.contains("Stay careful."));
        assert!(filtered.contains("Be brief."));
        assert!(!filtered.contains("# Working with subagents"));
        assert!(!filtered.contains("pending change"));
        assert!(!filtered.contains("give at most one short progress update"));
    }

    #[test]
    fn plan_workflow_survives_when_interactive_tool_is_disabled() {
        let filtered = filter_system_prompt(
            &super::super::prompt_plan::plan_mode_prompt(),
            &["plan_mode".to_string()],
        );

        assert!(filtered.contains("ask it clearly before plan_mode"));
        assert!(filtered.contains("backend closes Plan Mode automatically"));
        assert!(!filtered.contains("ask_user_choice"));
    }
}
