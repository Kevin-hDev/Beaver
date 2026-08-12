use tokio::process::Command;

#[cfg(all(test, windows))]
#[path = "tool_bash_platform_windows_tests.rs"]
mod windows_tests;

pub fn configure_process_group(command: &mut Command) {
    crate::services::process_tree::configure_tokio(command);
}

#[cfg(windows)]
pub fn powershell_executable() -> Result<std::path::PathBuf, String> {
    system32_file(&["WindowsPowerShell", "v1.0", "powershell.exe"])
        .ok_or_else(|| "Shell utilisateur indisponible.".to_string())
}

#[cfg(windows)]
fn system32_file(components: &[&str]) -> Option<std::path::PathBuf> {
    let root = std::env::var_os("SystemRoot")
        .map(std::path::PathBuf::from)
        .filter(|path| path.is_absolute())?;
    let mut path = root.join("System32");
    for component in components {
        if component.is_empty()
            || component.contains('/')
            || component.contains('\\')
            || *component == ".."
        {
            return None;
        }
        path.push(component);
    }
    path.is_file().then_some(path)
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
