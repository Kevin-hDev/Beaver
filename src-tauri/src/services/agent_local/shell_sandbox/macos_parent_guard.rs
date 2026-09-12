use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::process::Stdio;

const READY: u8 = 0x42;
const READY_TIMEOUT_MS: i32 = 2_000;
const WATCH_INTERVAL_MS: u64 = 100;

pub(super) fn prepare(
    shell: &OsStr,
    arguments: &[String],
    path: &OsStr,
) -> Result<super::launch::PreparedShellCommand, String> {
    let mut command = tokio::process::Command::new(super::launch::helper_executable()?);
    super::environment::protect_helper(&mut command);
    command
        .arg(super::helper::guard_arg())
        .arg("--")
        .arg(shell)
        .args(arguments)
        .env("PATH", path);
    Ok(super::launch::PreparedShellCommand {
        command,
        cleanup_dir: None,
    })
}

pub(super) fn run_guarded(arguments: Vec<OsString>) -> Result<i32, String> {
    let (executable, arguments) = super::helper::parse_guarded_command(arguments)?;
    install()?;
    let mut command = std::process::Command::new(executable);
    super::environment::protect_helper_std(&mut command);
    command.args(arguments);
    Err(command.exec().to_string())
}

pub(super) fn install() -> Result<(), String> {
    let parent_pid = u32::try_from(unsafe { libc::getppid() }).map_err(|_| error())?;
    let parent = crate::services::owned_process::OwnedProcess::identity(parent_pid)
        .map_err(|_| error())?;
    let root = crate::services::owned_process::OwnedProcess::identity(std::process::id())
        .map_err(|_| error())?;
    if root.native_scope != u64::from(root.pid) {
        return Err(error());
    }
    let executable = super::launch::helper_executable()?;
    let mut command = std::process::Command::new(executable);
    super::environment::protect_helper_std(&mut command);
    command
        .arg(super::helper::watchdog_arg())
        .args(identity_args(parent))
        .arg(root.pid.to_string())
        .arg(root.native_start_time.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = crate::services::owned_process::OwnedProcess::spawn(
        &mut command,
        crate::services::process_tree::ProcessKind::AgentShell,
    )
    .map_err(|_| error())?;
    let identity = match crate::services::owned_process::OwnedProcess::identity(child.id()) {
        Ok(identity) => identity,
        Err(_) => {
            crate::services::process_tree::terminate(
                &mut child,
                crate::services::process_tree::ProcessKind::AgentShell,
            );
            return Err(error());
        }
    };
    let ready = child.stdout.take().ok_or_else(error);
    if ready.and_then(wait_ready).is_err() {
        let _ = crate::services::owned_process::OwnedProcess::signal_exact(identity, true);
        let _ = child.wait();
        return Err(error());
    }
    Ok(())
}

pub(super) fn run_watchdog(arguments: Vec<OsString>) -> Result<i32, String> {
    let (parent, root_pid, root_start) = parse_watchdog(arguments)?;
    if crate::services::owned_process::OwnedProcess::identity(parent.pid).ok() != Some(parent) {
        return Err(error());
    }
    let root = inspect_root(root_pid, root_start)?.ok_or_else(error)?;
    if root.native_scope != u64::from(root.pid) {
        return Err(error());
    }
    std::io::stdout().write_all(&[READY]).map_err(|_| error())?;
    std::io::stdout().flush().map_err(|_| error())?;
    loop {
        let parent_alive =
            crate::services::owned_process::OwnedProcess::identity(parent.pid).ok() == Some(parent);
        let Some(current_root) = inspect_root(root_pid, root_start)? else {
            return Ok(0);
        };
        if !parent_alive {
            let _ = crate::services::owned_process::OwnedProcess::signal_exact(current_root, true);
            return Ok(0);
        }
        std::thread::sleep(std::time::Duration::from_millis(WATCH_INTERVAL_MS));
    }
}

fn wait_ready(output: std::process::ChildStdout) -> Result<(), String> {
    wait_ready_with_timeout(output, READY_TIMEOUT_MS)
}

fn wait_ready_with_timeout(
    mut output: std::process::ChildStdout,
    timeout_ms: i32,
) -> Result<(), String> {
    let mut descriptor = libc::pollfd {
        fd: output.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    let polled = unsafe { libc::poll(&mut descriptor, 1, timeout_ms) };
    if polled != 1 || descriptor.revents & libc::POLLIN == 0 {
        return Err(error());
    }
    let mut byte = [0_u8; 1];
    output.read_exact(&mut byte).map_err(|_| error())?;
    (byte == [READY]).then_some(()).ok_or_else(error)
}

fn inspect_root(
    pid: u32,
    start: u64,
) -> Result<Option<crate::services::owned_process::OwnedProcessIdentity>, String> {
    match crate::services::owned_process::OwnedProcess::inspect_for_recovery(pid, start) {
        Ok(crate::services::owned_process::OwnedProcessInspection::Owned(identity)) => {
            Ok(Some(identity))
        }
        Ok(crate::services::owned_process::OwnedProcessInspection::Unowned) => Ok(None),
        Err(_) if !crate::services::owned_process::OwnedProcess::process_exists(pid) => Ok(None),
        Err(_) => Err(error()),
    }
}

fn identity_args(
    identity: crate::services::owned_process::OwnedProcessIdentity,
) -> [String; 4] {
    [
        identity.pid.to_string(),
        identity.native_scope.to_string(),
        identity.native_start_time.to_string(),
        identity.executable.to_string(),
    ]
}

fn parse_watchdog(
    arguments: Vec<OsString>,
) -> Result<(crate::services::owned_process::OwnedProcessIdentity, u32, u64), String> {
    if arguments.len() != 6 {
        return Err(error());
    }
    let values = arguments
        .iter()
        .map(|value| value.to_str().ok_or_else(error))
        .collect::<Result<Vec<_>, _>>()?;
    let parent = crate::services::owned_process::OwnedProcessIdentity {
        pid: parse(values[0])?,
        native_scope: parse(values[1])?,
        native_start_time: parse(values[2])?,
        executable: parse(values[3])?,
    };
    Ok((parent, parse(values[4])?, parse(values[5])?))
}

fn parse<T: std::str::FromStr>(value: &str) -> Result<T, String> {
    if value.is_empty() || value.len() > 40 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(error());
    }
    value.parse().map_err(|_| error())
}

fn error() -> String {
    super::launch::sandbox_error()
}

#[cfg(test)]
#[path = "macos_parent_guard_tests.rs"]
mod tests;
