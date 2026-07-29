use std::path::Path;

use super::prompt_detailed_sections::{
    CAPABILITIES, CODE, GIT, SAFETY, STYLE, TOOLS, UNCERTAINTY, WEB_SEARCH,
};

pub fn build_with_behavior(
    working_dir: &Path,
    is_git: bool,
    git_root: Option<&Path>,
    behavior: Option<&str>,
) -> String {
    let identity = behavior.unwrap_or(IDENTITY);
    let style = operational_style(behavior.is_some());
    format!(
        "{identity}\n\n{}\n\n{CAPABILITIES}\n\n{}\n\n{TOOLS}\n\n{}\n\n{CODE}\n\n{GIT}\n\n{SAFETY}\n\n{WEB_SEARCH}\n\n{UNCERTAINTY}\n\n{}\n\n{style}",
        super::prompt_priority::PRIORITY,
        env_section(working_dir, is_git, git_root),
        super::subagent_parent_guidance::PARENT_GUIDANCE,
        super::prompt_external_content::EXTERNAL_CONTENT,
    )
}

fn operational_style(custom_behavior: bool) -> &'static str {
    if !custom_behavior {
        return STYLE;
    }
    STYLE
        .split_once("\n\n# Style")
        .map(|(operational, _)| operational)
        .unwrap_or(STYLE)
}

const IDENTITY: &str = "\
You are an autonomous coding agent with full access to the user's system through your tools.
You help users with software engineering tasks: writing code, debugging, managing files, \
running commands, searching the web, and more.
You are an agent, not a passive chatbot. You use tools to get things done, \
and you keep the user informed with short visible updates while you work.
You are highly capable and allow users to complete ambitious tasks that would otherwise be \
too complex or take too long.";

#[cfg(test)]
mod tests {
    use std::path::Path;

    /// Every `format!` argument here is a string, so a wrong argument order still compiles.
    /// This pins the positions instead.
    #[test]
    fn priority_precedes_the_rules_it_arbitrates() {
        let prompt = super::build_with_behavior(Path::new("."), false, None, None);

        let priority = prompt.find("# Priority order").expect("priority section");
        let capabilities = prompt.find("# Capabilities").expect("capabilities section");
        let safety = prompt.find("# Acting autonomously").expect("safety section");

        assert!(priority < capabilities);
        assert!(priority < safety);
    }
}

fn env_section(working_dir: &Path, is_git: bool, git_root: Option<&Path>) -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let shell = crate::services::env_detect::detect_shell();
    let os_version = crate::services::env_detect::detect_os_version();
    let date = chrono::Local::now().format("%Y-%m-%d");
    let git_flag = if is_git { "true" } else { "false" };
    let git_root_line = match git_root {
        Some(root) if root != working_dir => format!("\n - Git root: {}", root.display()),
        _ => String::new(),
    };
    format!(
        "# Environment\n\
         You have been invoked in the following environment:\n\
         - Primary working directory: {}\n\
         - Is a git repository: {git_flag}{git_root_line}\n\
         - Platform: {os} ({arch})\n\
         - Shell: {shell}\n\
         - OS Version: {os_version}\n\
         - Current date: {date}",
        working_dir.display()
    )
}
