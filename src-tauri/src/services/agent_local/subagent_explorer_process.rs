use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use zeroize::Zeroize;

use super::types_tools::ShellOutput;

const DEFAULT_TIMEOUT_SECS: u64 = 120;
const MAX_TIMEOUT_SECS: u64 = 600;
const MAX_OUTPUT_BYTES: usize = 64 * 1024;

pub async fn run(
    tokens: &[String],
    working_dir: &Path,
    timeout_secs: Option<u64>,
    cancel: CancellationToken,
) -> Result<ShellOutput, String> {
    if tokens.len() == 1 && tokens[0] == "pwd" {
        return current_directory(working_dir);
    }
    let (program, arguments) = tokens
        .split_first()
        .ok_or_else(|| "Commande d'exploration indisponible.".to_string())?;
    let executable = which::which(program)
        .ok()
        .and_then(|path| dunce::canonicalize(path).ok())
        .filter(|path| path.is_file())
        .ok_or_else(|| "Commande d'exploration indisponible.".to_string())?;
    let prepared =
        super::shell_sandbox::prepare_command(executable.as_os_str(), arguments, working_dir)?;
    let cleanup_dir = prepared.cleanup_dir;
    let mut command = prepared.command;
    command
        .current_dir(working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = match crate::services::owned_process::OwnedProcess::spawn_tokio(
        &mut command,
        crate::services::process_tree::ProcessKind::AgentShell,
    )
    .await
    {
        Ok(child) => child,
        Err(_) => {
            super::shell_sandbox::cleanup_temp(cleanup_dir).await;
            return Err("Commande d'exploration indisponible.".to_string());
        }
    };
    let Some(pid) = child.id() else {
        let _ = child.kill().await;
        super::shell_sandbox::cleanup_temp(cleanup_dir).await;
        return Err("Commande d'exploration indisponible.".to_string());
    };
    let pipes = (child.stdout.take(), child.stderr.take());
    let (Some(stdout), Some(stderr)) = pipes else {
        terminate(&mut child, pid).await;
        super::shell_sandbox::cleanup_temp(cleanup_dir).await;
        return Err("Commande d'exploration indisponible.".to_string());
    };
    let timeout = Duration::from_secs(
        timeout_secs
            .unwrap_or(DEFAULT_TIMEOUT_SECS)
            .min(MAX_TIMEOUT_SECS),
    );
    let outcome = {
        let execution = async {
            tokio::join!(
                super::tool_bash_io::read_bounded(stdout, MAX_OUTPUT_BYTES),
                super::tool_bash_io::read_bounded(stderr, MAX_OUTPUT_BYTES),
                child.wait()
            )
        };
        tokio::pin!(execution);
        tokio::select! {
            _ = cancel.cancelled() => ProcessOutcome::Cancelled,
            _ = tokio::time::sleep(timeout) => ProcessOutcome::TimedOut,
            result = &mut execution => ProcessOutcome::Finished(result),
        }
    };
    let sandbox_warning = cleanup_dir
        .as_deref()
        .and_then(super::shell_sandbox_diagnostics::warning);
    let mut result = match outcome {
        ProcessOutcome::Finished((Ok(stdout), Ok(stderr), Ok(status))) => {
            let (stdout, stdout_truncated) = render(stdout);
            let (stderr, stderr_truncated) = render(stderr);
            Ok(ShellOutput {
                stdout,
                stderr,
                exit_code: status.code().unwrap_or(-1),
                running: false,
                stopped: false,
                cancelled: false,
                blocked: false,
                timed_out: false,
                tracking_incomplete: false,
                output_truncated: stdout_truncated || stderr_truncated,
                output_incomplete: false,
                sandbox_warning: None,
                affected_paths: Vec::new(),
                file_changes: Vec::new(),
            })
        }
        ProcessOutcome::Finished((mut stdout, mut stderr, _)) => {
            clear_capture(&mut stdout);
            clear_capture(&mut stderr);
            Err("Commande d'exploration indisponible.".to_string())
        }
        ProcessOutcome::Cancelled => {
            terminate(&mut child, pid).await;
            Ok(interrupted("Commande d'exploration annulée.", false, true))
        }
        ProcessOutcome::TimedOut => {
            terminate(&mut child, pid).await;
            Ok(interrupted("Délai d'exploration dépassé.", true, false))
        }
    };
    if let Ok(output) = &mut result {
        output.sandbox_warning = sandbox_warning;
    }
    super::shell_sandbox::cleanup_temp(cleanup_dir).await;
    result
}

fn current_directory(working_dir: &Path) -> Result<ShellOutput, String> {
    let path = dunce::canonicalize(working_dir)
        .map_err(|_| "Commande d'exploration indisponible.".to_string())?;
    Ok(ShellOutput {
        stdout: path.to_string_lossy().into_owned(),
        stderr: String::new(),
        exit_code: 0,
        running: false,
        stopped: false,
        cancelled: false,
        blocked: false,
        timed_out: false,
        tracking_incomplete: false,
        output_truncated: false,
        output_incomplete: false,
        sandbox_warning: None,
        affected_paths: Vec::new(),
        file_changes: Vec::new(),
    })
}

type Captured = std::io::Result<(Vec<u8>, bool)>;

enum ProcessOutcome {
    Finished(
        (
            Captured,
            Captured,
            std::io::Result<std::process::ExitStatus>,
        ),
    ),
    Cancelled,
    TimedOut,
}

fn render((mut bytes, read_truncated): (Vec<u8>, bool)) -> (String, bool) {
    let text = String::from_utf8_lossy(&bytes).into_owned();
    bytes.zeroize();
    let (mut text, display_truncated) = super::tool_bash::truncate_output(&text);
    if read_truncated && !display_truncated {
        text.push_str("\n... [sortie tronquée]");
    }
    (text, read_truncated || display_truncated)
}

fn clear_capture(captured: &mut Captured) {
    if let Ok((bytes, _)) = captured {
        bytes.zeroize();
    }
}

async fn terminate(child: &mut tokio::process::Child, pid: u32) {
    super::tool_bash_platform::terminate_process_tree(pid).await;
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
}

fn interrupted(message: &str, timed_out: bool, cancelled: bool) -> ShellOutput {
    ShellOutput {
        stdout: String::new(),
        stderr: message.to_string(),
        exit_code: -1,
        running: false,
        stopped: false,
        cancelled,
        blocked: false,
        timed_out,
        tracking_incomplete: false,
        output_truncated: false,
        output_incomplete: false,
        sandbox_warning: None,
        affected_paths: Vec::new(),
        file_changes: Vec::new(),
    }
}

#[cfg(test)]
#[path = "subagent_explorer_process_tests.rs"]
mod tests;
