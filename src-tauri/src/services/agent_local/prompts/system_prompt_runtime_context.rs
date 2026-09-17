use std::path::Path;

pub fn agentic_environment(
    working_dir: &Path,
    is_git: bool,
    git_root: Option<&Path>,
) -> String {
    let git_root_line = match git_root {
        Some(root) if root != working_dir => format!("\n - Git root: {}", root.display()),
        _ => String::new(),
    };
    environment(
        working_dir,
        &format!("\n - Is a git repository: {is_git}{git_root_line}"),
    )
}

pub fn chatbot_environment(working_dir: &Path) -> String {
    environment(working_dir, "")
}

fn environment(working_dir: &Path, extra: &str) -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let shell = crate::services::env_detect::detect_shell();
    let os_version = crate::services::env_detect::detect_os_version();
    let date = chrono::Local::now().format("%Y-%m-%d");
    format!(
        "# Environment\n\
         - Working directory: {}{extra}\n\
         - Platform: {os} ({arch})\n\
         - Shell: {shell}\n\
         - OS Version: {os_version}\n\
         - Current date: {date}",
        working_dir.display()
    )
}
