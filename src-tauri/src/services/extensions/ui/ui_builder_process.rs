use std::ffi::OsString;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const BUILD_TIMEOUT: Duration = Duration::from_secs(60);

pub(super) fn run(
    runtime: &super::ui_builder::UiBuildRuntime,
    arguments: &[OsString],
    temporary: &Path,
    cancelled: impl Fn() -> bool,
    owner: Option<&super::install_jobs::InstallControl>,
    reset: impl Fn() -> Result<(), String>,
) -> Result<Vec<u8>, super::OperationFailure> {
    use super::process_runner::ProcessFailure;
    let mut first = true;
    let mut remaining = BUILD_TIMEOUT;
    super::install_retry::run(
        &mut remaining,
        |timeout| {
            if !first {
                reset().map_err(|_| ProcessFailure::Failed)?;
            }
            first = false;
            let path = super::process_environment::inherited_path()
                .map_err(|_| ProcessFailure::EnvironmentInvalid)?;
            let mut command = Command::new(&runtime.node);
            command.args(arguments).current_dir(&runtime.directory);
            super::process_environment::configure_installer(&mut command, path, temporary)
                .map_err(|_| ProcessFailure::EnvironmentInvalid)?;
            super::installer_process::run(
                command,
                timeout,
                || cancelled() || owner.is_some_and(|owner| owner.producer_should_stop()),
                |identity| {
                    owner.map_or(Ok(()), |owner| {
                        super::install_signal::InstallSignal::process_started(owner, identity)
                    })
                },
                || {
                    owner.map_or(Ok(()), |owner| {
                        super::install_signal::InstallSignal::process_stopped(owner)
                    })
                },
            )
        },
        || {
            if cancelled() {
                return Err(ProcessFailure::Interrupted);
            }
            owner.map_or(Ok(false), |owner| {
                owner
                    .after_producer_stopped()
                    .map_err(|_| ProcessFailure::Interrupted)
            })
        },
    )
    .map_err(|error| match error {
        super::process_runner::ProcessFailure::StopUnconfirmed => {
            super::OperationFailure::CleanupFailed
        }
        _ => super::OperationFailure::InstallFailed,
    })
}
