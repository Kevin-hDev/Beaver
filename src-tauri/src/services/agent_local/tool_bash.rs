use crate::services::agent_local::security;
use crate::services::agent_local::types_tools::ShellOutput;
use rand::RngCore;
use std::path::Path;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

const MAX_LINES: usize = 2000;
const MAX_BYTES: usize = 50 * 1024;
const DEFAULT_TIMEOUT: u64 = 120;
const MAX_TIMEOUT: u64 = 600;

pub async fn execute_shell(
    command: &str,
    working_dir: &Path,
    timeout_secs: Option<u64>,
) -> Result<ShellOutput, String> {
    if let Err(reason) = security::check_destructive_command(command) {
        return Ok(ShellOutput {
            stdout: String::new(),
            stderr: reason,
            exit_code: -1,
            timed_out: false,
            new_cwd: None,
            affected_paths: Vec::new(),
            file_changes: Vec::new(),
        });
    }

    let before = super::tool_bash_changes::snapshot(working_dir);
    if super::tool_bash_long::should_run_in_background(command) {
        let output =
            super::tool_bash_long::execute_background_shell(command, working_dir, timeout_secs)
                .await?;
        return Ok(with_changed_paths(output, working_dir, before));
    }

    let secs = timeout_secs.unwrap_or(DEFAULT_TIMEOUT).min(MAX_TIMEOUT);
    let (shell, flag) = detect_shell();
    let marker = generate_cwd_marker();
    let wrapped = wrap_command_with_cwd(command, &marker);

    let child = Command::new(&shell)
        .args([&flag, &wrapped])
        .current_dir(working_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Erreur lancement shell: {e}"))?;

    let output = timeout(Duration::from_secs(secs), child.wait_with_output()).await;

    match output {
        Ok(Ok(out)) => {
            let raw_stdout = String::from_utf8_lossy(&out.stdout);
            let (user_output, new_cwd) = extract_cwd(&raw_stdout, &marker);
            let stdout = truncate_output(&user_output);
            let raw_stderr = String::from_utf8_lossy(&out.stderr);
            let stderr = truncate_output(&strip_marker(&raw_stderr, &marker));
            Ok(ShellOutput {
                stdout,
                stderr,
                exit_code: out.status.code().unwrap_or(-1),
                timed_out: false,
                new_cwd,
                affected_paths: Vec::new(),
                file_changes: Vec::new(),
            })
            .map(|output| with_changed_paths(output, working_dir, before))
        }
        Ok(Err(e)) => Err(format!("Erreur exécution: {e}")),
        Err(_) => Ok(ShellOutput {
            stdout: String::new(),
            stderr: format!("Timeout après {secs}s"),
            exit_code: -1,
            timed_out: true,
            new_cwd: None,
            affected_paths: Vec::new(),
            file_changes: Vec::new(),
        })
        .map(|output| with_changed_paths(output, working_dir, before)),
    }
}

fn with_changed_paths(
    mut output: ShellOutput,
    working_dir: &Path,
    before: super::tool_bash_changes::FileSnapshot,
) -> ShellOutput {
    let after = super::tool_bash_changes::snapshot(working_dir);
    output.file_changes = super::tool_bash_changes::changes(&before, &after);
    output.affected_paths = output
        .file_changes
        .iter()
        .map(|change| change.path.clone())
        .collect();
    output
}

fn generate_cwd_marker() -> String {
    let mut bytes = [0u8; 8];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("<<CWD_MARKER_{hex}>>")
}

fn wrap_command_with_cwd(command: &str, marker: &str) -> String {
    if cfg!(target_os = "windows") {
        format!(
            "{command} ; $clgoStatus = if ($?) {{ 0 }} else {{ 1 }} ; Write-Output '{marker}' ; (Get-Location).Path ; exit $clgoStatus"
        )
    } else {
        format!("{command}\nclgo_status=$?\necho '{marker}'\npwd -P\nexit $clgo_status")
    }
}

fn strip_marker(text: &str, marker: &str) -> String {
    text.replace(marker, "")
}

fn extract_cwd(raw_stdout: &str, marker: &str) -> (String, Option<String>) {
    if let Some(idx) = raw_stdout.find(marker) {
        let user_output = raw_stdout[..idx].trim_end_matches('\n').to_string();
        let after = raw_stdout[idx + marker.len()..].trim();
        let new_cwd = if !after.is_empty() {
            let p = Path::new(after);
            if p.is_absolute() && p.is_dir() {
                Some(after.to_string())
            } else {
                None
            }
        } else {
            None
        };
        (user_output, new_cwd)
    } else {
        (raw_stdout.to_string(), None)
    }
}

pub(super) fn detect_shell() -> (String, String) {
    if cfg!(target_os = "windows") {
        ("powershell".to_string(), "-Command".to_string())
    } else {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        (shell, "-c".to_string())
    }
}

pub(super) fn truncate_output(output: &str) -> String {
    let mut result = String::new();
    for (line_count, line) in output.lines().enumerate() {
        if line_count >= MAX_LINES || result.len() + line.len() > MAX_BYTES {
            result.push_str("\n... [tronqué]");
            break;
        }
        if line_count > 0 {
            result.push('\n');
        }
        result.push_str(line);
    }
    result
}

#[cfg(test)]
#[path = "tool_bash_tests.rs"]
mod tests;
