pub fn plan_mode_prompt() -> String {
    format!(
        "\
<critical_plan_mode_workflow>
You are in Plan Mode. Follow this workflow exactly and in order.
This Plan Mode workflow overrides the general interactive-choice guidance.

<mandatory_steps>
1. Explore the project with read-only tools when code context is useful.
2. If you need an important user answer before publishing the plan, ask it clearly before plan_mode.
3. Call plan_mode only when the final plan is ready.
4. plan_mode asks the user for final approval itself and waits for the answer.
5. If the user approves, the backend closes Plan Mode automatically and tells you to start implementation.
6. If the user requests adjustments, revise the plan and publish the updated version.
7. If the user dismisses approval, the current turn stops and Plan Mode remains enabled.
</mandatory_steps>

<allowed_actions>
Use only these read-only or Plan Mode tools: {}.
</allowed_actions>

<blocked_actions>
Keep the codebase unchanged while Plan Mode is active. The backend blocks write tools and todo_write, then unlocks them automatically after approval.
</blocked_actions>
</critical_plan_mode_workflow>",
        super::tool_plan_guard::PLAN_MODE_ALLOWED_ACTIONS_TEXT
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn plan_prompt_uses_strict_workflow_markers() {
        let prompt = super::plan_mode_prompt();
        assert!(prompt.contains("<critical_plan_mode_workflow>"));
        assert!(prompt.contains("<mandatory_steps>"));
        assert!(prompt.contains("<allowed_actions>"));
        assert!(prompt.contains("<blocked_actions>"));
        assert!(prompt.contains("Follow this workflow exactly and in order"));
        assert!(prompt.contains("ask it clearly before plan_mode"));
        assert!(prompt.contains("plan_mode asks the user for final approval itself"));
        assert!(prompt.contains("backend closes Plan Mode automatically"));
        assert!(!prompt.contains("exitplan_mode"));
    }

    #[test]
    fn plan_prompt_lists_guard_allowed_tools() {
        let prompt = super::plan_mode_prompt();
        for tool in super::super::tool_plan_guard::PLAN_MODE_ALLOWED_TOOL_NAMES {
            assert!(prompt.contains(tool), "missing {tool}");
        }
    }
}
