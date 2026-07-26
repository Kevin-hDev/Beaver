use super::manifest::{base_requirements, expected_requirements, source_requirements};
use super::paths::{
    commit_staged_runtime, remove_if_exists, runtime_paths, runtime_ready_at, write_stamp,
};
use super::{cache, command};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeInstallStep {
    CreatingEnvironment,
    PreparingInstaller,
    InstallingDependencies,
    Finalizing,
}

pub(super) fn prepare_runtime(
    sidecar_dir: &Path,
    family_id: &str,
    cancel: &CancellationToken,
    on_progress: &dyn Fn(RuntimeInstallStep),
) -> Result<PathBuf, String> {
    let requirements = expected_requirements(sidecar_dir, family_id)?;
    let base_requirements = base_requirements(sidecar_dir, family_id)?;
    let source_requirements = source_requirements(family_id)?;
    let paths = runtime_paths(sidecar_dir, family_id)?;
    if runtime_ready_at(&paths.live, &requirements) {
        remove_if_exists(&paths.staging)?;
        remove_if_exists(&paths.backup)?;
        return Ok(paths.python_in(&paths.live));
    }
    remove_if_exists(&paths.staging)?;
    let result = (|| {
        check_cancel(cancel)?;
        on_progress(RuntimeInstallStep::CreatingEnvironment);
        let python = find_python()?;
        run_cancellable(
            Command::new(python)
                .args(["-m", "venv"])
                .arg(&paths.staging),
            cancel,
            "Création du runtime Forecast impossible",
        )?;
        let staged_python = paths.python_in(&paths.staging);
        on_progress(RuntimeInstallStep::PreparingInstaller);
        let cache = cache::prepare(sidecar_dir)?;
        let manifest = paths.staging.join("requirements.txt");
        std::fs::write(&manifest, &base_requirements)
            .map_err(|_| "Préparation du runtime Forecast impossible".to_string())?;
        on_progress(RuntimeInstallStep::InstallingDependencies);
        let mut install = Command::new(&staged_python);
        install
            .args([
                "-m",
                "pip",
                "install",
                "--no-input",
                "--require-hashes",
                "--only-binary=:all:",
                "--index-url",
                "https://pypi.org/simple",
                "-r",
            ])
            .arg(&manifest);
        command::configure_pip(&mut install, &cache);
        command::run_cancellable(
            &mut install,
            cancel,
            "Installation du moteur Forecast impossible",
        )?;
        if let Some(source) = source_requirements.as_deref() {
            install_audited_source(&staged_python, &paths.staging, source, &cache, cancel)?;
        }
        check_cancel(cancel)?;
        write_stamp(&paths.staging, &requirements)?;
        if !runtime_ready_at(&paths.staging, &requirements) {
            return Err("Validation du runtime Forecast impossible".to_string());
        }
        on_progress(RuntimeInstallStep::Finalizing);
        check_cancel(cancel)?;
        commit_staged_runtime(&paths)?;
        Ok(paths.python_in(&paths.live))
    })();
    if result.is_err() {
        let _ = remove_if_exists(&paths.staging);
    }
    result
}

fn install_audited_source(
    python: &Path,
    staging: &Path,
    requirements: &str,
    cache: &Path,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let manifest = staging.join("audited-source.txt");
    std::fs::write(&manifest, requirements)
        .map_err(|_| "Préparation du moteur Forecast impossible".to_string())?;
    let mut install = Command::new(python);
    install
        .args([
            "-m",
            "pip",
            "install",
            "--no-input",
            "--require-hashes",
            "--no-build-isolation",
            "--no-deps",
            "-r",
        ])
        .arg(manifest);
    command::configure_pip(&mut install, cache);
    command::run_cancellable(
        &mut install,
        cancel,
        "Installation du moteur Forecast impossible",
    )
}

fn check_cancel(cancel: &CancellationToken) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("cancelled".to_string())
    } else {
        Ok(())
    }
}

fn find_python() -> Result<PathBuf, String> {
    for candidate in ["python3.12", "python3", "python"] {
        if let Ok(path) = which::which(candidate) {
            if python_version_supported(&path) {
                return Ok(path);
            }
        }
    }
    Err("Runtime Python compatible indisponible".to_string())
}

fn python_version_supported(path: &Path) -> bool {
    Command::new(path)
        .args([
            "-c",
            "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')",
        ])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .is_some_and(|version| is_supported_version(&version))
}

fn is_supported_version(version: &str) -> bool {
    version.trim() == "3.12"
}

fn run_cancellable(
    command: &mut Command,
    cancel: &CancellationToken,
    message: &str,
) -> Result<(), String> {
    command::harden_python(command);
    command::run_cancellable(command, cancel, message)
}

#[cfg(test)]
mod tests {
    use super::is_supported_version;

    #[test]
    fn runtime_python_is_fixed_to_3_12() {
        assert!(is_supported_version("3.12\n"));
        assert!(!is_supported_version("3.13"));
    }
}
