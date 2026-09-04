use std::ffi::OsString;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const BUILD_TIMEOUT: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_millis(50);
const MAX_STDOUT_BYTES: usize = 1_048_576;
const MAX_STDERR_BYTES: usize = 65_536;

struct BoundedOutput {
    bytes: Vec<u8>,
    overflow: bool,
}

pub(super) fn run(
    runtime: &super::ui_builder::UiBuildRuntime,
    arguments: &[OsString],
    temporary: &Path,
    cancelled: impl Fn() -> bool,
) -> Result<Vec<u8>, super::OperationFailure> {
    if cancelled() {
        return Err(super::OperationFailure::InstallFailed);
    }
    let path = super::process_environment::inherited_path()
        .map_err(|_| super::OperationFailure::EnvironmentInvalid)?;
    let mut command = Command::new(&runtime.node);
    command
        .args(arguments)
        .current_dir(&runtime.directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    super::process_environment::configure_installer(&mut command, path, temporary)
        .map_err(|_| super::OperationFailure::EnvironmentInvalid)?;
    let mut child = crate::services::owned_process::OwnedProcess::spawn(
        &mut command,
        crate::services::process_tree::ProcessKind::ExtensionInstaller,
    )
    .map_err(|_| super::OperationFailure::RuntimeUnavailable)?;
    let pid = child.id();
    let stdout = child
        .stdout
        .take()
        .ok_or(super::OperationFailure::InstallFailed)?;
    let stderr = child
        .stderr
        .take()
        .ok_or(super::OperationFailure::InstallFailed)?;
    let stdout_reader = std::thread::spawn(move || drain(stdout, MAX_STDOUT_BYTES));
    let stderr_reader = std::thread::spawn(move || drain(stderr, MAX_STDERR_BYTES));
    let deadline = Instant::now() + BUILD_TIMEOUT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if !cancelled() && Instant::now() < deadline => {
                std::thread::sleep(POLL_INTERVAL);
            }
            _ => {
                crate::services::process_tree::terminate(
                    &mut child,
                    crate::services::process_tree::ProcessKind::ExtensionInstaller,
                );
                break Err(super::OperationFailure::InstallFailed);
            }
        }
    };
    crate::services::owned_process::release(pid);
    let stdout = collect(stdout_reader)?;
    let stderr = collect(stderr_reader)?;
    let status = status?;
    if !status.success() || stdout.overflow || stderr.overflow {
        return Err(super::OperationFailure::InstallFailed);
    }
    Ok(stdout.bytes)
}

fn collect(
    reader: std::thread::JoinHandle<std::io::Result<BoundedOutput>>,
) -> Result<BoundedOutput, super::OperationFailure> {
    reader
        .join()
        .ok()
        .and_then(Result::ok)
        .ok_or(super::OperationFailure::InstallFailed)
}

fn drain(mut reader: impl Read, limit: usize) -> std::io::Result<BoundedOutput> {
    let mut bytes = Vec::with_capacity(limit.min(8_192));
    let mut overflow = false;
    let mut chunk = [0_u8; 8_192];
    loop {
        let read = reader.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        let available = limit.saturating_sub(bytes.len());
        bytes.extend_from_slice(&chunk[..read.min(available)]);
        overflow |= read > available;
    }
    Ok(BoundedOutput { bytes, overflow })
}
