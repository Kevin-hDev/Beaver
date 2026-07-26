use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use super::command::{regular_program, run_status, spawn_background, CommandSpec};
use super::health::HealthToken;
use super::verify::Installation;
use super::WorkerError;

const INSTALL_TIMEOUT: Duration = Duration::from_secs(15 * 60);

pub(crate) fn apply(asset: &Path, current: &Installation) -> Result<(), WorkerError> {
    let health = HealthToken::generate(crate::services::paths::data_dir())?;
    let install = install_spec(asset);
    if !regular_program(&install.program) {
        return Err(WorkerError);
    }
    run_status(&install, INSTALL_TIMEOUT)?;
    let mut child = spawn_background(&launch_spec(current, health.value()))?;
    if health.wait().is_err() {
        let _ = child.kill();
        let _ = child.wait();
        return Err(WorkerError);
    }
    std::fs::remove_file(asset).map_err(|_| WorkerError)
}

pub(crate) fn install_spec(asset: &Path) -> CommandSpec {
    CommandSpec::new(
        "/usr/bin/pkexec",
        vec![
            OsString::from("/usr/bin/apt-get"),
            OsString::from("install"),
            OsString::from("-y"),
            asset.as_os_str().to_owned(),
        ],
    )
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
#[path = "linux_tests.rs"]
mod tests;
