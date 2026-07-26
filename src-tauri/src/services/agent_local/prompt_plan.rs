pub fn plan_mode_prompt() -> String {
    format!(
        "\
<critical_plan_mode_workflow>
You are in Plan Mode. Follow this workflow exactly and in order.
This Plan Mode workflow overrides the general interactive-choice guidance.

<mandatory_steps>
1. Explore the project with read-only tools when code context is useful.
2. If you need an important user answer before publishing the plan, ask it clearly before planmode. You may use normal assistant text or ask_user_choice.
3. Call planmode only when the final plan is ready.
4. planmode asks the user for final approval itself and returns the decision.
5. If planmode returns implement, call exitplanmode with status approved.
6. After exitplanmode approved succeeds, immediately start implementation without waiting for another user message.
7. If planmode returns continue_planning, continue Plan Mode and publish an updated plan.
8. If planmode returns quit_plan, call exitplanmode with status rejected.
</mandatory_steps>

<allowed_actions>
Use only these read-only or Plan Mode tools: {}.
</allowed_actions>

<blocked_actions>
Keep the codebase unchanged until exitplanmode approved succeeds. The backend blocks write tools and todo_write while Plan Mode is active.
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
        assert!(prompt.contains("normal assistant text or ask_user_choice"));
        assert!(prompt.contains("planmode asks the user for final approval itself"));
        assert!(prompt.contains("If planmode returns implement"));
    }

    #[test]
    fn plan_prompt_lists_guard_allowed_tools() {
        let prompt = super::plan_mode_prompt();
        for tool in super::super::tool_plan_guard::PLAN_MODE_ALLOWED_TOOL_NAMES {
            assert!(prompt.contains(tool), "missing {tool}");
        }
    }
}
