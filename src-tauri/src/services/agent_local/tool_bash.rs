use crate::services::agent_local::security;
use crate::services::agent_local::types_tools::ShellOutput;
use std::path::{Path, PathBuf};
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
    let prepared_command = prepare_command(command);

    let child = Command::new(&shell)
        .args([&flag, &prepared_command])
        .current_dir(working_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Erreur lancement shell: {e}"))?;

    let output = timeout(Duration::from_secs(secs), child.wait_with_output()).await;

    match output {
        Ok(Ok(out)) => {
            let stdout = truncate_output(&String::from_utf8_lossy(&out.stdout));
            let stderr = truncate_output(&String::from_utf8_lossy(&out.stderr));
            Ok(ShellOutput {
                stdout,
                stderr,
                exit_code: out.status.code().unwrap_or(-1),
                timed_out: false,
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

fn prepare_command(command: &str) -> String {
    if cfg!(target_os = "windows") {
        format!(
            "{command} ; $clgoStatus = if ($?) {{ 0 }} else {{ 1 }} ; exit $clgoStatus"
        )
    } else {
        command.to_string()
    }
}

pub(crate) fn resolve_workdir(
    requested: Option<&str>,
    project_root: &Path,
) -> Result<PathBuf, String> {
    let candidate = match requested.map(str::trim).filter(|path| !path.is_empty()) {
        Some(path) => {
            if path.len() > 4_096 || path.contains('\0') || !Path::new(path).is_absolute() {
                return Err("Le workdir Bash doit être un chemin absolu valide.".to_string());
            }
            PathBuf::from(path)
        }
        None => project_root.to_path_buf(),
    };
    if !candidate.is_dir() {
        return Err("Le workdir Bash est inaccessible.".to_string());
    }
    candidate
        .canonicalize()
        .map_err(|_| "Le workdir Bash est inaccessible.".to_string())
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
