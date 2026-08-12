use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::services::work_registry::ServiceWorkCancellation;

const MAX_ARGUMENTS: usize = 48;
const MAX_ARGUMENT_CHARS: usize = 4_096;
const MAX_PATH_CHARS: usize = 16_384;
#[cfg(windows)]
const MAX_SYSTEM_ROOT_CHARS: usize = 1_024;
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
    let path = std::env::var_os("PATH")
        .filter(|value| valid_environment_value(value, MAX_PATH_CHARS))
        .ok_or(ProcessFailure::EnvironmentInvalid)?;
    let mut command = Command::new(program);
    command
        .args(arguments)
        .current_dir(working_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    configure_environment(&mut command, path, temporary_directory)?;
    crate::services::process_tree::configure(&mut command);
    let mut child = command.spawn().map_err(|_| ProcessFailure::Unavailable)?;
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

fn configure_environment(
    command: &mut Command,
    path: OsString,
    temporary_directory: &Path,
) -> Result<(), ProcessFailure> {
    command
        .env_clear()
        .env("PATH", path)
        .env("HOME", temporary_directory)
        .env("TMPDIR", temporary_directory)
        .env("TMP", temporary_directory)
        .env("TEMP", temporary_directory);
    #[cfg(windows)]
    {
        command.env("USERPROFILE", temporary_directory);
        if let Some(system_root) = std::env::var_os("SystemRoot") {
            if !valid_environment_value(&system_root, MAX_SYSTEM_ROOT_CHARS) {
                return Err(ProcessFailure::EnvironmentInvalid);
            }
            command.env("SystemRoot", system_root);
        }
    }
    Ok(())
}

fn valid_environment_value(value: &std::ffi::OsStr, maximum: usize) -> bool {
    let text = value.to_string_lossy();
    text.chars().count() <= maximum && !text.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_exit::AppExitCoordinator;
    use crate::services::extensions::work_supervision::ExtensionWorkServices;

    #[test]
    fn inherited_environment_is_bounded_and_single_line() {
        assert!(valid_environment_value(
            std::ffi::OsStr::new("/usr/bin:/bin"),
            MAX_PATH_CHARS
        ));
        assert!(!valid_environment_value(
            std::ffi::OsStr::new("/usr/bin\nunsafe"),
            MAX_PATH_CHARS
        ));
        assert!(!valid_environment_value(
            std::ffi::OsStr::new(&"a".repeat(MAX_PATH_CHARS + 1)),
            MAX_PATH_CHARS
        ));
    }

    #[test]
    fn accepts_a_realistic_long_developer_path() {
        let path = (0..69)
            .map(|index| format!("/tmp/beaver-developer-path/{index:055}"))
            .collect::<Vec<_>>()
            .join(":");

        assert!(path.chars().count() > MAX_ARGUMENT_CHARS);
        assert!(valid_environment_value(
            std::ffi::OsStr::new(&path),
            MAX_PATH_CHARS
        ));
    }

    #[test]
    fn child_home_is_isolated_inside_the_temporary_workspace() {
        let temporary = tempfile::tempdir().unwrap();
        let mut command = Command::new("unused");

        configure_environment(
            &mut command,
            OsString::from("isolated-path"),
            temporary.path(),
        )
        .unwrap();

        let environment = command
            .get_envs()
            .map(|(key, value)| (key.to_owned(), value.map(std::ffi::OsStr::to_owned)))
            .collect::<std::collections::HashMap<_, _>>();
        assert_eq!(
            environment.get(std::ffi::OsStr::new("HOME")),
            Some(&Some(temporary.path().as_os_str().to_owned()))
        );
        #[cfg(windows)]
        assert_eq!(
            environment.get(std::ffi::OsStr::new("USERPROFILE")),
            Some(&Some(temporary.path().as_os_str().to_owned()))
        );
    }

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
