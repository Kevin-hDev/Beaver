use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::command::{run_status, CommandSpec};
use super::health::HealthToken;
use super::macos_bundle::{validate_beaver_source, validate_current, ValidatedBundle};
use super::macos_mount::MountedDmg;
use super::macos_process::terminate_matching;
use super::macos_swap::{InstallTransaction, StagedBundle};
use super::verify::Installation;
use super::WorkerError;

const OPEN_TIMEOUT: Duration = Duration::from_secs(15);

pub(crate) fn apply(asset: &Path, current: &Installation) -> Result<(), WorkerError> {
    let current_path = current.bundle.as_deref().ok_or(WorkerError)?;
    let current_bundle = validate_current(current_path)?;
    if std::fs::canonicalize(&current.executable).map_err(|_| WorkerError)?
        != current_bundle.executable
    {
        return Err(WorkerError);
    }
    let health = HealthToken::generate(crate::services::paths::data_dir())?;
    let stage = match prepare_stage(asset, &current_bundle) {
        Ok(stage) => stage,
        Err(error) => {
            let _ = restart_previous(&current_bundle.root);
            return Err(error);
        }
    };
    let mut transaction = match InstallTransaction::begin(&current_bundle, stage) {
        Ok(transaction) => transaction,
        Err(error) => {
            let _ = restart_previous(&current_bundle.root);
            return Err(error);
        }
    };
    let installed = match validate_beaver_source(transaction.installed_bundle()) {
        Ok(bundle) => bundle,
        Err(error) => {
            if transaction.rollback().is_ok() {
                let _ = restart_previous(transaction.previous_bundle());
            }
            return Err(error);
        }
    };
    let launch = run_status(&launch_spec(&installed.root, health.value()), OPEN_TIMEOUT)
        .and_then(|()| health.wait());
    if launch.is_err() {
        return rollback_failed_launch(transaction, &installed, health.value());
    }
    transaction.commit()?;
    std::fs::remove_file(asset).map_err(|_| WorkerError)
}

fn prepare_stage(asset: &Path, current: &ValidatedBundle) -> Result<StagedBundle, WorkerError> {
    let mounted = MountedDmg::attach(asset)?;
    let source = validate_beaver_source(&mounted.root().join("Beaver.app"))?;
    let installation_root = current.root.parent().ok_or(WorkerError)?;
    let stage = StagedBundle::copy(&source, installation_root)?;
    mounted.detach()?;
    Ok(stage)
}

fn rollback_failed_launch(
    mut transaction: InstallTransaction,
    installed: &ValidatedBundle,
    token: &str,
) -> Result<(), WorkerError> {
    if terminate_matching(&installed.executable, token).is_err() {
        transaction.abandon();
        return Err(WorkerError);
    }
    let previous = transaction.previous_bundle().to_path_buf();
    let rollback = transaction.rollback();
    if rollback.is_ok() {
        let _ = restart_previous(&previous);
    }
    Err(WorkerError)
}

pub(crate) fn launch_spec(bundle: &Path, token: &str) -> CommandSpec {
    CommandSpec::new(
        "/usr/bin/open",
        vec![
            OsString::from("-n"),
            bundle.as_os_str().to_owned(),
            OsString::from("--args"),
            OsString::from(crate::services::update_health::UPDATE_HEALTH_ARG),
            OsString::from(token),
        ],
    )
}

fn restart_previous(bundle: &Path) -> Result<(), WorkerError> {
    run_status(&restart_spec(bundle), OPEN_TIMEOUT)
}

pub(crate) fn restart_spec(bundle: &Path) -> CommandSpec {
    CommandSpec::new(
        PathBuf::from("/usr/bin/open"),
        vec![OsString::from("-n"), bundle.as_os_str().to_owned()],
    )
}

#[cfg(test)]
#[path = "macos_tests.rs"]
mod tests;
