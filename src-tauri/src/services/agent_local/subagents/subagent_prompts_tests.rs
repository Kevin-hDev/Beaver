#[cfg(test)]
mod tests {
    use crate::services::agent_local::subagent_prompts::compose_for_test;
    use crate::services::agent_local::subagent_tool_profile::SubagentToolProfile;

    #[test]
    fn test_explorer_system_contains_role() {
        let prompt = explorer_prompt();
        assert!(
            prompt.contains("<role>"),
            "Le prompt explorer doit contenir la balise <role>"
        );
    }

    #[test]
    fn test_explorer_system_contains_date() {
        let prompt = explorer_prompt();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert!(
            prompt.contains(&today),
            "Le prompt explorer doit contenir la date du jour ({today})"
        );
    }

    #[test]
    fn test_explorer_system_contains_platform() {
        let prompt = explorer_prompt();
        let os = std::env::consts::OS;
        assert!(
            prompt.contains(os),
            "Le prompt explorer doit contenir la plateforme OS ({os})"
        );
    }

    #[test]
    fn test_explorer_system_contains_truthfulness_rule() {
        let prompt = explorer_prompt();
        assert!(prompt.contains("Never invent files"));
    }

    #[test]
    fn test_coder_system_contains_constraints() {
        assert!(coder_prompt().contains("<constraints>"));
    }

    #[test]
    fn test_coder_system_contains_security_and_truthfulness_rules() {
        let prompt = coder_prompt();
        assert!(prompt.contains("Never invent files"));
        assert!(prompt.contains("Fail closed on security errors"));
    }

    #[test]
    fn prompts_only_describe_tools_from_their_profile() {
        let explorer = explorer_prompt();
        assert!(explorer.contains("- bash:"));
        assert!(!explorer.contains("write_file"));
        assert!(!explorer.contains("load_skill"));
        let coder = coder_prompt();
        assert!(coder.contains("write_file"));
        assert!(coder.contains("load_skill"));
        assert!(!coder.contains("delegate_task"));
        assert!(!coder.contains("forecast"));
    }

    #[test]
    fn prompt_injects_runtime_context_once_and_uses_configured_language() {
        let prompt = compose_for_test(
            SubagentToolProfile::Coder,
            std::path::Path::new("/tmp/worktree-authoritative"),
            true,
            "Français",
            Some("AGENTS-CONTEXT\n\nPERSONALITY-CONTEXT".into()),
            &[("review".into(), "Review code".into())],
        );
        assert_eq!(prompt.matches("AGENTS-CONTEXT").count(), 1);
        assert_eq!(prompt.matches("PERSONALITY-CONTEXT").count(), 1);
        assert!(prompt.contains("/tmp/worktree-authoritative"));
        assert!(prompt.contains("You MUST respond in Français"));
        assert!(prompt.contains("Every report heading must use Français"));
    }

    fn explorer_prompt() -> String {
        compose_for_test(
            SubagentToolProfile::Explorer,
            std::path::Path::new("/tmp/project"),
            false,
            "Français",
            None,
            &[],
        )
    }

    fn coder_prompt() -> String {
        compose_for_test(
            SubagentToolProfile::Coder,
            std::path::Path::new("/tmp/worktree"),
            true,
            "Français",
            None,
            &[("review".into(), "Review code".into())],
        )
    }
}
