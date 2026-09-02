use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::services::work_registry::ServiceWorkCancellation;

const MAX_ARGUMENTS: usize = 48;
const MAX_ARGUMENT_CHARS: usize = 4_096;
const POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessFailure {
    CommandInvalid,
    EnvironmentInvalid,
    Unavailable,
    Failed,
    Timeout,
    Interrupted,
}

pub fn run(
    program: &Path,
    arguments: &[OsString],
    working_directory: &Path,
    temporary_directory: &Path,
    timeout: Duration,
    cancellation: &ServiceWorkCancellation,
) -> Result<(), ProcessFailure> {
    if !program.is_absolute()
        || !program.is_file()
        || !working_directory.is_absolute()
        || !working_directory.is_dir()
        || !temporary_directory.is_absolute()
        || !temporary_directory.is_dir()
        || arguments.len() > MAX_ARGUMENTS
        || arguments
            .iter()
            .any(|argument| argument.to_string_lossy().chars().count() > MAX_ARGUMENT_CHARS)
    {
        return Err(ProcessFailure::CommandInvalid);
    }
    if cancellation.is_cancelled() {
        return Err(ProcessFailure::Interrupted);
    }
    let path = super::process_environment::inherited_path()
        .map_err(|_| ProcessFailure::EnvironmentInvalid)?;
    let mut command = Command::new(program);
    command
        .args(arguments)
        .current_dir(working_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    super::process_environment::configure_installer(&mut command, path, temporary_directory)
        .map_err(|_| ProcessFailure::EnvironmentInvalid)?;
    let mut child = crate::services::owned_process::OwnedProcess::spawn(
        &mut command,
        crate::services::process_tree::ProcessKind::ExtensionInstaller,
    )
    .map_err(|_| ProcessFailure::Unavailable)?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(_)) => return Err(ProcessFailure::Failed),
            Ok(None) if cancellation.is_cancelled() => {
                crate::services::process_tree::terminate(
                    &mut child,
                    crate::services::process_tree::ProcessKind::ExtensionInstaller,
                );
                return Err(ProcessFailure::Interrupted);
            }
            Ok(None) if Instant::now() < deadline => std::thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                crate::services::process_tree::terminate(
                    &mut child,
                    crate::services::process_tree::ProcessKind::ExtensionInstaller,
                );
                return Err(ProcessFailure::Timeout);
            }
            Err(_) => {
                crate::services::process_tree::terminate(
                    &mut child,
                    crate::services::process_tree::ProcessKind::ExtensionInstaller,
                );
                return Err(ProcessFailure::Interrupted);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_exit::AppExitCoordinator;
    use crate::services::extensions::work_supervision::ExtensionWorkServices;

    #[tokio::test]
    async fn cancellation_terminates_and_reaps_a_real_installer_child() {
        let node = which::which("node").unwrap().canonicalize().unwrap();
        let workspace = tempfile::tempdir().unwrap();
        let script = workspace.path().join("wait.mjs");
        std::fs::write(&script, "setInterval(() => {}, 1000);\n").unwrap();
        let temporary = workspace.path().join("tmp");
        std::fs::create_dir(&temporary).unwrap();
        let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
        let work = ExtensionWorkServices::new(coordinator.work_supervisor());
        let admission = work.try_admit_operation().expect("operation admission");
        let cancellation = admission.cancellation();
        let root = workspace.path().to_path_buf();
        let arguments = vec![script.into_os_string()];

        let child = tokio::task::spawn_blocking(move || {
            run(
                &node,
                &arguments,
                &root,
                &temporary,
                Duration::from_secs(30),
                &cancellation,
            )
        });
        tokio::time::sleep(Duration::from_millis(150)).await;
        work.begin_closing();

        let result = tokio::time::timeout(Duration::from_secs(3), child)
            .await
            .expect("cancelled installer must finish")
            .expect("installer task must not panic");
        assert_eq!(result, Err(ProcessFailure::Interrupted));
        drop(admission);
    }
}
