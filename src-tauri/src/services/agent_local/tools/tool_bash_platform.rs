use tokio::process::Command;

#[cfg(all(test, windows))]
#[path = "tool_bash_platform_windows_tests.rs"]
mod windows_tests;

pub fn configure_process_group(command: &mut Command) {
    crate::services::process_tree::configure_tokio(command);
}

#[cfg(windows)]
pub fn powershell_executable() -> Result<std::path::PathBuf, String> {
    crate::services::system_executable::powershell()
        .map_err(|_| "Shell utilisateur indisponible.".to_string())
}

pub async fn terminate_process_tree(pid: u32) {
    let _ = tokio::task::spawn_blocking(move || {
        crate::services::process_tree::kill(
            pid,
            crate::services::process_tree::ProcessKind::AgentShell,
        );
    })
    .await;
}
