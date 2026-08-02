use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;

use super::tool_bash_progress::ShellProgress;
use super::types_tools::ShellOutput;

#[cfg(not(windows))]
const MAX_COMMAND_BYTES: usize = 512 * 1024;
#[cfg(windows)]
const MAX_COMMAND_BYTES: usize = 24 * 1024;
const MAX_INPUT_BYTES: usize = 64 * 1024;
const INPUT_WRITE_TIMEOUT_SECS: u64 = 5;
pub const DEFAULT_YIELD_MS: u64 = 10_000;
const MIN_YIELD_MS: u64 = 250;
const MAX_YIELD_MS: u64 = 30_000;
const MAX_LINES: usize = 2_000;
const MAX_BYTES: usize = 50 * 1024;

pub struct ShellExecutionContext<'a> {
    pub owner_session_id: &'a str,
    pub hard_timeout_secs: Option<u64>,
    pub yield_time_ms: Option<u64>,
    pub cancel: CancellationToken,
    pub progress: Option<ShellProgress>,
}

pub async fn execute_shell_managed(
    command: &str,
    working_dir: &Path,
    context: ShellExecutionContext<'_>,
) -> Result<ShellOutput, String> {
    validate_command(command)?;
    if let Err(reason) = super::security::check_destructive_command(command) {
        return Ok(super::tool_bash_result::blocked(reason));
    }
    if context.hard_timeout_secs == Some(0) {
        return Err("Le timeout shell doit etre superieur a zero.".to_string());
    }

    let session = super::tool_bash_process::spawn(super::tool_bash_process::SpawnRequest {
        command,
        working_dir,
        owner_session_id: context.owner_session_id,
        hard_timeout_secs: context.hard_timeout_secs,
        progress: context.progress,
        agent_cancel: context.cancel.clone(),
    })
    .await?;
    let snapshot = super::tool_bash_wait::wait(
        &session,
        yield_duration(context.yield_time_ms),
        &context.cancel,
        true,
    )
    .await;
    session.set_progress(None);
    let finished = !snapshot.running;
    let output = super::tool_bash_result::from_snapshot(&session, snapshot);
    if finished {
        super::tool_bash_registry::remove(session.id());
    }
    Ok(output)
}

pub async fn control_shell_session(
    process_id: &str,
    input: Option<&str>,
    eof: bool,
    stop: bool,
    owner_session_id: &str,
    yield_time_ms: Option<u64>,
    cancel: CancellationToken,
    progress: Option<ShellProgress>,
) -> Result<(ShellOutput, super::tool_bash_registry::RegisteredCommand), String> {
    if let Some(input) = input {
        validate_input(input)?;
    }
    let (session, command) = super::tool_bash_registry::get(process_id, owner_session_id)?;
    if !stop {
        if let Some(input) = input.filter(|value| !value.is_empty()) {
            if let Err(reason) = super::security::check_destructive_command(input) {
                return Ok((super::tool_bash_result::blocked(reason), command));
            }
        }
    }
    session.set_progress(progress);
    if stop || input.is_some_and(|value| value.contains('\u{3}')) {
        session.stop();
    } else if let Some(input) = input {
        if !input.is_empty() {
            if let Err(error) = write_session_input(&session, input, &cancel).await {
                session.set_progress(None);
                return Err(error);
            }
        }
    }
    if eof {
        session.close_stdin().await;
    }
    let snapshot = super::tool_bash_wait::wait(
        &session,
        yield_duration(yield_time_ms),
        &cancel,
        true,
    )
    .await;
    session.set_progress(None);
    let output = super::tool_bash_result::from_snapshot(&session, snapshot);
    if session.is_done() {
        super::tool_bash_registry::remove(session.id());
    }
    Ok((output, command))
}

async fn write_session_input(
    session: &super::tool_bash_session::ShellSession,
    input: &str,
    caller_cancel: &CancellationToken,
) -> Result<(), String> {
    tokio::select! {
        result = session.write_input(input.as_bytes()) => result,
        _ = caller_cancel.cancelled() => {
            session.cancel();
            Err("Ecriture vers le shell annulee.".to_string())
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(INPUT_WRITE_TIMEOUT_SECS)) => {
            session.cancel();
            Err("Délai d'écriture vers le shell dépassé.".to_string())
        }
    }
}

#[cfg(test)]
pub async fn execute_shell(
    command: &str,
    working_dir: &Path,
    timeout_secs: Option<u64>,
) -> Result<ShellOutput, String> {
    let owner = uuid::Uuid::new_v4().to_string();
    execute_shell_managed(
        command,
        working_dir,
        ShellExecutionContext {
            owner_session_id: &owner,
            hard_timeout_secs: timeout_secs,
            yield_time_ms: None,
            cancel: CancellationToken::new(),
            progress: None,
        },
    )
    .await
}

pub(crate) fn resolve_workdir(
    requested: Option<&str>,
    project_root: &Path,
) -> Result<PathBuf, String> {
    let candidate = match requested.map(str::trim).filter(|path| !path.is_empty()) {
        Some(path) => {
            if path.len() > 4_096 || path.contains('\0') || !Path::new(path).is_absolute() {
                return Err("Le workdir Bash doit etre un chemin absolu valide.".to_string());
            }
            PathBuf::from(path)
        }
        None => project_root.to_path_buf(),
    };
    if !candidate.is_dir() {
        return Err("Le workdir Bash est inaccessible.".to_string());
    }
    let candidate = dunce::canonicalize(candidate)
        .map_err(|_| "Le workdir Bash est inaccessible.".to_string())?;
    super::directory_access::ensure_allowed(&candidate)
}

pub(super) fn truncate_output(output: &str) -> (String, bool) {
    let mut result = String::new();
    for (line_count, line) in output.lines().enumerate() {
        if line_count >= MAX_LINES || result.len() + line.len() > MAX_BYTES {
            result.push_str("\n... [sortie tronquée]");
            return (result, true);
        }
        if line_count > 0 {
            result.push('\n');
        }
        result.push_str(line);
    }
    (result, false)
}

pub(crate) fn validate_command(command: &str) -> Result<(), String> {
    if command.trim().is_empty() || command.len() > MAX_COMMAND_BYTES || command.contains('\0') {
        return Err("Commande shell invalide.".to_string());
    }
    Ok(())
}

pub(crate) fn validate_input(input: &str) -> Result<(), String> {
    if input.len() > MAX_INPUT_BYTES || input.contains('\0') {
        return Err("Entree shell invalide.".to_string());
    }
    Ok(())
}

fn yield_duration(value: Option<u64>) -> std::time::Duration {
    std::time::Duration::from_millis(
        value
            .unwrap_or(DEFAULT_YIELD_MS)
            .clamp(MIN_YIELD_MS, MAX_YIELD_MS),
    )
}

#[cfg(test)]
#[path = "tool_bash_test_modules.rs"]
mod tests;
