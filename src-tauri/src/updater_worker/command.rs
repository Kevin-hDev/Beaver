use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use super::WorkerError;

const POLL_INTERVAL: Duration = Duration::from_millis(50);
const MAX_COMMAND_ARGUMENTS: usize = 12;
const MAX_ARGUMENT_LENGTH: usize = 4096;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct CommandSpec {
    pub(crate) program: PathBuf,
    pub(crate) args: Vec<OsString>,
}

impl CommandSpec {
    pub(crate) fn new(program: impl Into<PathBuf>, args: Vec<OsString>) -> Self {
        Self {
            program: program.into(),
            args,
        }
    }
}

pub(crate) fn run_status(spec: &CommandSpec, timeout: Duration) -> Result<(), WorkerError> {
    validate_spec(spec)?;
    let mut child = command(spec)
        .stdout(Stdio::null())
        .spawn()
        .map_err(|_| WorkerError)?;
    wait_success(&mut child, timeout)
}

pub(crate) fn spawn_background(spec: &CommandSpec) -> Result<Child, WorkerError> {
    validate_spec(spec)?;
    command(spec)
        .stdout(Stdio::null())
        .spawn()
        .map_err(|_| WorkerError)
}

pub(crate) fn run_bounded_output(
    spec: &CommandSpec,
    timeout: Duration,
    max_output: usize,
) -> Result<Vec<u8>, WorkerError> {
    validate_spec(spec)?;
    if max_output == 0 || max_output > 1024 * 1024 {
        return Err(WorkerError);
    }
    let mut child = command(spec)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|_| WorkerError)?;
    let stdout = child.stdout.take().ok_or(WorkerError)?;
    let reader = std::thread::spawn(move || {
        let mut output = Vec::with_capacity(max_output.min(8192));
        stdout
            .take((max_output + 1) as u64)
            .read_to_end(&mut output)
            .map(|_| output)
    });
    let status = wait_status(&mut child, timeout);
    let output = reader
        .join()
        .map_err(|_| WorkerError)?
        .map_err(|_| WorkerError)?;
    if !status?.success() || output.len() > max_output {
        return Err(WorkerError);
    }
    Ok(output)
}

fn command(spec: &CommandSpec) -> Command {
    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .stdin(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command
}

fn wait_success(child: &mut Child, timeout: Duration) -> Result<(), WorkerError> {
    if wait_status(child, timeout)?.success() {
        Ok(())
    } else {
        Err(WorkerError)
    }
}

fn wait_status(
    child: &mut Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus, WorkerError> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|_| WorkerError)? {
            return Ok(status);
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(WorkerError);
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

fn validate_spec(spec: &CommandSpec) -> Result<(), WorkerError> {
    if !spec.program.is_absolute()
        || spec.args.len() > MAX_COMMAND_ARGUMENTS
        || spec.args.iter().any(|argument| {
            argument
                .to_string_lossy()
                .chars()
                .take(MAX_ARGUMENT_LENGTH + 1)
                .count()
                > MAX_ARGUMENT_LENGTH
        })
    {
        return Err(WorkerError);
    }
    Ok(())
}

pub(crate) fn regular_program(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
}

#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;
