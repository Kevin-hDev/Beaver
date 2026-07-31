use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

pub async fn build_command(
    command: &str,
    working_dir: &Path,
    owner_session_id: &str,
) -> Result<Command, String> {
    let shell = user_shell()?;
    let prepared = super::tool_bash_profile::prepare(
        owner_session_id,
        &shell,
        working_dir,
        command,
    )
    .await;
    let arguments = shell_arguments(&prepared);
    let mut process = Command::new(shell);
    process
        .args(arguments)
        .current_dir(working_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
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
        {
            return Ok(shell);
        }
    }
    Err("Shell utilisateur indisponible.".to_string())
}

#[cfg(windows)]
fn user_shell() -> Result<String, String> {
    Ok("powershell".to_string())
}

#[cfg(unix)]
fn shell_arguments(command: &str) -> Vec<String> {
    vec!["-c".to_string(), command.to_string()]
}

#[cfg(windows)]
fn shell_arguments(command: &str) -> Vec<String> {
    let prepared = format!(
        "{command}\n$beaverStatus = if ($?) {{ 0 }} else {{ 1 }} ; exit $beaverStatus"
    );
    vec!["-Command".to_string(), prepared]
}
