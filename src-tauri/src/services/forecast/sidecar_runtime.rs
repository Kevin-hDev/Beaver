use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;

#[path = "sidecar_runtime_cache.rs"]
mod cache;
#[path = "sidecar_runtime_command.rs"]
mod command;
#[path = "sidecar_runtime_install.rs"]
mod install;
#[path = "sidecar_runtime_lock_data.rs"]
mod lock_data;
#[path = "sidecar_runtime_manifest.rs"]
mod manifest;
#[path = "sidecar_runtime_paths.rs"]
mod paths;

pub use install::RuntimeInstallStep;
use manifest::expected_requirements;
pub(crate) use manifest::{locked_package_version, runtime_id};
use paths::runtime_paths;
#[cfg(test)]
use paths::{commit_staged_runtime, RuntimePaths};

pub fn ensure_runtime(sidecar_dir: &Path, family_id: &str) -> Result<PathBuf, String> {
    let requirements = expected_requirements(sidecar_dir, family_id)?;
    let paths = runtime_paths(sidecar_dir, family_id)?;
    if paths::runtime_ready_at(&paths.live, &requirements) {
        return Ok(paths.python_in(&paths.live));
    }
    Err("Runtime Forecast non préparé".to_string())
}

pub fn prepare_runtime<F>(
    sidecar_dir: &Path,
    family_id: &str,
    cancel: &CancellationToken,
    on_progress: F,
) -> Result<PathBuf, String>
where
    F: Fn(RuntimeInstallStep),
{
    install::prepare_runtime(sidecar_dir, family_id, cancel, &on_progress)
}

pub fn family_runtime_ready(sidecar_dir: &Path, family_id: &str) -> bool {
    ensure_runtime(sidecar_dir, family_id).is_ok()
}

pub fn remove_family_runtime(sidecar_dir: &Path, family_id: &str) -> Result<(), String> {
    let paths = runtime_paths(sidecar_dir, family_id)?;
    paths::remove_runtime_paths(&paths)
}

#[path = "sidecar_runtime_tests.rs"]
#[cfg(test)]
mod tests;
