use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
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
    let (program, arguments) = tokens
        .split_first()
        .ok_or_else(|| "Commande d'exploration indisponible.".to_string())?;
    let mut command = Command::new(program);
    command
        .args(arguments)
        .current_dir(working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    super::tool_bash_platform::configure_process_group(&mut command);
    let mut child = command
        .spawn()
        .map_err(|_| "Commande d'exploration indisponible.".to_string())?;
    let Some(pid) = child.id() else {
        let _ = child.kill().await;
        return Err("Commande d'exploration indisponible.".to_string());
    };
    let pipes = (child.stdout.take(), child.stderr.take());
    let (Some(stdout), Some(stderr)) = pipes else {
        terminate(&mut child, pid).await;
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
    match outcome {
        ProcessOutcome::Finished((Ok(stdout), Ok(stderr), Ok(status))) => Ok(ShellOutput {
            stdout: render(stdout),
            stderr: render(stderr),
            exit_code: status.code().unwrap_or(-1),
            running: false,
            timed_out: false,
            affected_paths: Vec::new(),
            file_changes: Vec::new(),
        }),
        ProcessOutcome::Finished((mut stdout, mut stderr, _)) => {
            clear_capture(&mut stdout);
            clear_capture(&mut stderr);
            Err("Commande d'exploration indisponible.".to_string())
        }
        ProcessOutcome::Cancelled => {
            terminate(&mut child, pid).await;
            Ok(interrupted("Commande d'exploration annulée.", false))
        }
        ProcessOutcome::TimedOut => {
            terminate(&mut child, pid).await;
            Ok(interrupted("Délai d'exploration dépassé.", true))
        }
    }
}

type Captured = std::io::Result<(Vec<u8>, bool)>;

enum ProcessOutcome {
    Finished((Captured, Captured, std::io::Result<std::process::ExitStatus>)),
    Cancelled,
    TimedOut,
}

fn render((mut bytes, exceeded): (Vec<u8>, bool)) -> String {
    let mut text = String::from_utf8_lossy(&bytes).into_owned();
    bytes.zeroize();
    text = super::tool_bash::truncate_output(&text);
    if exceeded {
        text.push_str("\n... [sortie tronquée]");
    }
    text
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

fn interrupted(message: &str, timed_out: bool) -> ShellOutput {
    ShellOutput {
        stdout: String::new(),
        stderr: message.to_string(),
        exit_code: -1,
        running: false,
        timed_out,
        affected_paths: Vec::new(),
        file_changes: Vec::new(),
    }
}

#[cfg(test)]
#[path = "subagent_explorer_process_tests.rs"]
mod tests;
