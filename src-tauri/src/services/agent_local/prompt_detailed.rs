use std::path::Path;

use super::prompt_detailed_sections::{
    CAPABILITIES, CODE, ERRORS, GIT, HONESTY, SAFETY, STYLE, TOOLS, VERIFICATION, WEB_SEARCH,
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
        "{identity}\n\n{CAPABILITIES}\n\n{}\n\n{TOOLS}\n\n{}\n\n{CODE}\n\n{GIT}\n\n{SAFETY}\n\n{ERRORS}\n\n{WEB_SEARCH}\n\n{HONESTY}\n\n{VERIFICATION}\n\n{style}",
        env_section(working_dir, is_git, git_root),
        super::subagent_parent_guidance::PARENT_GUIDANCE,
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
