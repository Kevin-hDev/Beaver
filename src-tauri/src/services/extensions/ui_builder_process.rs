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
) -> Result<Vec<u8>, super::OperationFailure> {
    let path = super::process_environment::inherited_path()
        .map_err(|_| super::OperationFailure::EnvironmentInvalid)?;
    let mut command = Command::new(&runtime.node);
    command.args(arguments).current_dir(&runtime.directory);
    super::process_environment::configure_installer(&mut command, path, temporary)
        .map_err(|_| super::OperationFailure::EnvironmentInvalid)?;
    super::installer_process::run(
        command,
        BUILD_TIMEOUT,
        cancelled,
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
    .map_err(|error| match error {
        super::process_runner::ProcessFailure::StopUnconfirmed => {
            super::OperationFailure::CleanupFailed
        }
        _ => super::OperationFailure::InstallFailed,
    })
}
