//! One synchronous owner drives scoped async process IO without detached reader threads.
use super::process_runner::ProcessFailure;
use crate::services::{owned_process::OwnedProcess, process_tree};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const POLL: Duration = Duration::from_millis(25);
const STOP_BUDGET: Duration = Duration::from_secs(2);
const MAX_OUTPUT: usize = 1_048_576;

pub(super) fn run(
    mut command: std::process::Command,
    timeout: Duration,
    cancelled: impl Fn() -> bool,
    started: impl Fn(crate::services::owned_process::OwnedProcessIdentity) -> Result<(), ()>,
    stopped: impl Fn() -> Result<(), ()>,
) -> Result<Vec<u8>, ProcessFailure> {
    command = gated(command)?;
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| ProcessFailure::Unavailable)?;
    runtime.block_on(async {
        if cancelled() { return Err(ProcessFailure::Interrupted); }
        let kind = process_tree::ProcessKind::ExtensionInstaller;
        let mut command = tokio::process::Command::from(command);
        let (mut child, scope) = OwnedProcess::spawn_tokio_scoped(&mut command, kind).await
            .map_err(|_| ProcessFailure::Unavailable)?;
        let pid = child.id().ok_or(ProcessFailure::Unavailable)?;
        let mut stdout = child.stdout.take().ok_or(ProcessFailure::Unavailable)?;
        let identity = OwnedProcess::identity(pid).map_err(|_| ProcessFailure::Unavailable);
        let admitted = identity.and_then(|identity| started(identity).map_err(|_| ProcessFailure::Unavailable));
        let mut owner_pipe = child.stdin.take();
        let released = if admitted.is_ok() && !cancelled() {
            match owner_pipe.as_mut() {
                Some(input) => input.write_all(&[1]).await.map_err(|_| ProcessFailure::Failed),
                None => Err(ProcessFailure::Failed),
            }
        } else { Err(ProcessFailure::Interrupted) };
        let deadline = Instant::now() + timeout;
        let mut bytes = Vec::new();
        let mut chunk = [0_u8; 8192];
        let mut eof = false;
        let result = loop {
            if released.is_err() { break Err(ProcessFailure::Interrupted); }
            if cancelled() { break Err(ProcessFailure::Interrupted); }
            if Instant::now() >= deadline { break Err(ProcessFailure::Timeout); }
            match child.try_wait() {
                Ok(Some(status)) => break if status.success() { Ok(()) } else { Err(ProcessFailure::Failed) },
                Err(_) => break Err(ProcessFailure::Failed),
                Ok(None) => {}
            }
            tokio::select! {
                read = stdout.read(&mut chunk), if !eof => {
                    match read {
                        Ok(0) => eof = true,
                        Ok(read) if bytes.len() + read <= MAX_OUTPUT => bytes.extend_from_slice(&chunk[..read]),
                        _ => break Err(ProcessFailure::Failed),
                    }
                }
                _ = tokio::time::sleep(POLL) => {}
            }
        };
        // Reap and prove the entire scope empty even when the root exited successfully.
        // Descendants can keep writing files or hold stdout after their parent exits.
        if !process_tree::terminate_tokio_scoped(&mut child, kind, &scope, pid,
            Instant::now() + STOP_BUDGET).await {
            return Err(ProcessFailure::StopUnconfirmed);
        }
        drop(owner_pipe);
        stopped().map_err(|_| ProcessFailure::StopUnconfirmed)?;
        result?;
        if !eof {
            let remaining = MAX_OUTPUT.saturating_sub(bytes.len());
            let mut tail = Vec::new();
            let read = tokio::time::timeout(STOP_BUDGET,
                stdout.take(remaining as u64 + 1).read_to_end(&mut tail)).await;
            match read {
                Ok(Ok(_)) if tail.len() <= remaining => bytes.extend_from_slice(&tail),
                _ => return Err(ProcessFailure::Failed),
            }
        }
        Ok(bytes)
    })
}

// Until the private journal contains native identity, stdin is the launch barrier.
// A crashed parent closes it: npm/UI are never imported before the single go byte.
const GATE: &str = include_str!("installer_process_gate.cjs");

fn gated(command: std::process::Command) -> Result<std::process::Command, ProcessFailure> {
    let mut gated = std::process::Command::new(command.get_program());
    gated.args(["--eval", GATE, "--"]).args(command.get_args());
    if let Some(directory) = command.get_current_dir() {
        gated.current_dir(directory);
    }
    gated.env_clear();
    for (key, value) in command.get_envs() {
        match value {
            Some(value) => {
                gated.env(key, value);
            }
            None => {
                gated.env_remove(key);
            }
        }
    }
    Ok(gated)
}

#[cfg(test)]
#[path = "installer_process_death_tests.rs"]
mod death_tests;
#[cfg(test)]
#[path = "installer_process_tests.rs"]
mod tests;
