use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use super::command::{run_status, spawn_background, CommandSpec};
use super::health::HealthToken;
use super::verify::Installation;
use super::WorkerError;

const INSTALL_TIMEOUT: Duration = Duration::from_secs(15 * 60);

pub(crate) fn apply(asset: &Path, current: &Installation) -> Result<(), WorkerError> {
    let health = HealthToken::generate(crate::services::paths::data_dir())?;
    run_status(
        &install_spec(asset, &current.working_directory),
        INSTALL_TIMEOUT,
    )?;
    let mut child = spawn_background(&launch_spec(current, health.value()))?;
    if health.wait().is_err() {
        let _ = child.kill();
        let _ = child.wait();
        return Err(WorkerError);
    }
    std::fs::remove_file(asset).map_err(|_| WorkerError)
}

pub(crate) fn install_spec(asset: &Path, installation: &Path) -> CommandSpec {
    let mut destination = OsString::from("/D=");
    destination.push(installation.as_os_str());
    CommandSpec::new(asset, vec![OsString::from("/S"), destination])
}

fn launch_spec(current: &Installation, token: &str) -> CommandSpec {
    CommandSpec::new(
        &current.executable,
        vec![
            OsString::from(crate::services::update_health::UPDATE_HEALTH_ARG),
            OsString::from(token),
        ],
    )
}

#[cfg(test)]
#[path = "windows_tests.rs"]
mod tests;
