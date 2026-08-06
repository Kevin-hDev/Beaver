use std::path::Path;

use super::prompt_detailed_sections::{CODE, GIT, SAFETY, STYLE, TOOLS, UNCERTAINTY, WEB_SEARCH};

pub fn build_with_behavior(
    _working_dir: &Path,
    _is_git: bool,
    _git_root: Option<&Path>,
    behavior: Option<&str>,
) -> String {
    if let Some(custom) = behavior {
        return custom.to_string();
    }
    format!(
        "{IDENTITY}\n\n{}\n\n{}\n\n{SAFETY}\n\n{CODE}\n\n{GIT}\n\n{TOOLS}\n\n{WEB_SEARCH}\n\n{}\n\n{}\n\n{UNCERTAINTY}\n\n{}\n\n{STYLE}",
        super::prompt_objective::DONE,
        super::prompt_priority::PRIORITY,
        super::subagent_parent_guidance::PARENT_GUIDANCE,
        super::prompt_objective::WORKFLOW,
        super::prompt_external_content::EXTERNAL_CONTENT,
    )
}

const IDENTITY: &str = "\
You are an autonomous coding agent with access to the user's system through your tools.
You help users with software engineering tasks: writing code, debugging, managing files, \
running commands, searching the web, and more.
You are an agent, not a passive chatbot. You use tools to get things done, \
and you keep the user informed with short visible updates while you work.";

#[cfg(test)]
mod tests {
    use std::path::Path;

    /// Every `format!` argument here is a string, so a wrong argument order still compiles.
    /// This pins the positions and keeps `# Style` last.
    #[test]
    fn sections_follow_the_reference_structure() {
        let prompt = super::build_with_behavior(Path::new("."), false, None, None);

        let expected = [
            "# What done means",
            "# Priority order",
            "# Acting autonomously",
            "# Working with code",
            "# Working with git",
            "# Using your tools",
            "# Web search",
            "# Working with subagents",
            "# How you work",
            "# When you are not sure",
            "# External content",
            "# Style",
        ];

        let mut previous = 0;
        for heading in expected {
            let at = prompt
                .find(heading)
                .unwrap_or_else(|| panic!("missing section: {heading}"));
            assert!(at > previous, "section out of order: {heading}");
            previous = at;
        }
    }
}
