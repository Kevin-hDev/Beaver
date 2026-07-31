use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

pub async fn build_command(
    command: &str,
    working_dir: &Path,
    owner_session_id: &str,
) -> Result<Command, String> {
    let shell = user_shell()?;
    let profile = super::tool_bash_profile::prepare(owner_session_id, &shell, working_dir).await;
    let arguments = shell_arguments(command);
    let mut process = Command::new(shell);
    process
        .args(arguments)
        .current_dir(working_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if let Some(profile) = profile {
        profile.apply(&mut process);
    }
    super::tool_bash_platform::configure_process_group(&mut process);
    Ok(process)
}

#[cfg(unix)]
fn user_shell() -> Result<String, String> {
    let mut candidates = std::env::var("SHELL").into_iter().collect::<Vec<_>>();
    #[cfg(target_os = "macos")]
    candidates.push("/bin/zsh".to_string());
    candidates.push("/bin/bash".to_string());
    candidates.push("/bin/sh".to_string());
    for shell in candidates {
        let path = Path::new(&shell);
        if shell.len() <= 4_096
            && !shell.contains('\0')
            && path.is_absolute()
            && path.is_file()
            && super::tool_bash_profile::supports_shell(&shell)
        {
            return Ok(shell);
        }
    }
    Err("Shell utilisateur indisponible.".to_string())
}

#[cfg(windows)]
fn user_shell() -> Result<String, String> {
    Ok(super::tool_bash_platform::powershell_executable()?
        .to_string_lossy()
        .to_string())
}

#[cfg(unix)]
pub(super) fn shell_arguments(command: &str) -> Vec<String> {
    let wrapper = format!(
        "if [ \"${{{}+x}}\" = x ]; then eval \"${{{}-}}${{{}-}}\"; unset {} {}; fi; set +e; eval \"$1\"; beaver_status=$?; wait; exit \"$beaver_status\"",
        super::tool_bash_profile::SNAPSHOT_ENVS[0],
        super::tool_bash_profile::SNAPSHOT_ENVS[0],
        super::tool_bash_profile::SNAPSHOT_ENVS[1],
        super::tool_bash_profile::SNAPSHOT_ENVS[0],
        super::tool_bash_profile::SNAPSHOT_ENVS[1],
    );
    vec![
        "-c".to_string(),
        wrapper,
        "beaver-shell".to_string(),
        command.to_string(),
    ]
}

#[cfg(windows)]
pub(super) fn shell_arguments(command: &str) -> Vec<String> {
    vec![
        "-NoLogo".to_string(),
        "-NoProfile".to_string(),
        "-NonInteractive".to_string(),
        "-Command".to_string(),
        powershell_script(command),
    ]
}

#[cfg(any(windows, test))]
fn powershell_script(command: &str) -> String {
    format!(
        "$global:LASTEXITCODE = $null\n{command}\n$beaverSucceeded = $?; $beaverStatus = $global:LASTEXITCODE; if ($beaverSucceeded) {{ exit 0 }}; if ($null -ne $beaverStatus -and [int]$beaverStatus -ne 0) {{ exit [int]$beaverStatus }}; exit 1"
    )
}

#[cfg(test)]
#[path = "tool_bash_shell_tests.rs"]
mod tests;
