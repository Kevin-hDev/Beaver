use crate::services::work_registry::ServiceWorkCancellation;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

use super::python_runtime::PythonRuntime;
use super::runtime_error::RuntimeError;
use super::wheels::Wheelhouse;

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;

const STAMP: &str = ".runtime.stamp";

pub async fn ensure_runtime(
    source: &Path,
    cancel: &ServiceWorkCancellation,
) -> Result<PathBuf, String> {
    ensure(source, cancel).await.map_err(|error| {
        log::warn!("[searxng] runtime category={}", error.category());
        error.public_message().to_string()
    })
}

async fn ensure(source: &Path, cancel: &ServiceWorkCancellation) -> Result<PathBuf, RuntimeError> {
    validate_source(source)?;
    let wheelhouse =
        super::wheels::for_source(source)?.ok_or(RuntimeError::WheelhouseUnavailable)?;
    let base_python = PythonRuntime::resolve(&wheelhouse.manifest).await?;
    let venv = super::paths::venv_dir();
    let python = venv_python(&venv);
    let venv_python = base_python.with_program(python.clone());
    if !python.exists() {
        let mut command = base_python.command();
        command.args(["-m", "venv"]).arg(&venv);
        run(command, cancel).await?;
    }

    let stamp_path = venv.join(STAMP);
    let stamp = source_stamp(source)?;
    let installed = std::fs::read_to_string(&stamp_path).unwrap_or_default();
    if installed != stamp {
        install_build_tools(&venv_python, &wheelhouse, cancel).await?;
        install_requirements(&venv_python, source, &wheelhouse, cancel).await?;
        if cancel.is_cancelled() {
            return Err(RuntimeError::Cancelled);
        }
        std::fs::write(stamp_path, stamp).map_err(|_| RuntimeError::EnvironmentUnavailable)?;
    }
    Ok(python)
}

fn validate_source(source: &Path) -> Result<(), RuntimeError> {
    let required = ["setup.py", "requirements.txt", "LICENSE", "searx/webapp.py"];
    if required.iter().all(|file| source.join(file).exists()) {
        Ok(())
    } else {
        Err(RuntimeError::WheelhouseUnavailable)
    }
}

fn source_stamp(source: &Path) -> Result<String, RuntimeError> {
    let mut hasher = Sha256::new();
    for file in ["setup.py", "requirements.txt", "searx/version.py"] {
        let body =
            std::fs::read(source.join(file)).map_err(|_| RuntimeError::WheelhouseUnavailable)?;
        hasher.update(body);
    }
    Ok(hex::encode(hasher.finalize()))
}

async fn install_build_tools(
    python: &PythonRuntime,
    wheelhouse: &Wheelhouse,
    cancel: &ServiceWorkCancellation,
) -> Result<(), RuntimeError> {
    let mut command = python.command();
    command
        .args(["-m", "pip", "install", "--no-index", "--find-links"])
        .arg(&wheelhouse.path)
        .args(["setuptools", "wheel"]);
    run(command, cancel).await
}

async fn install_requirements(
    python: &PythonRuntime,
    source: &Path,
    wheelhouse: &Wheelhouse,
    cancel: &ServiceWorkCancellation,
) -> Result<(), RuntimeError> {
    let mut command = python.command();
    command
        .args(["-m", "pip", "install", "--no-index", "--find-links"])
        .arg(&wheelhouse.path)
        .arg("-r")
        .arg(source.join("requirements.txt"));
    run(command, cancel).await
}

async fn run(mut command: Command, cancel: &ServiceWorkCancellation) -> Result<(), RuntimeError> {
    command
        .env("PIP_DISABLE_PIP_VERSION_CHECK", "1")
        .env("PIP_NO_INPUT", "1")
        .env("PYTHONUNBUFFERED", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = crate::services::owned_process::OwnedProcess::spawn_tokio(
        &mut command,
        crate::services::process_tree::ProcessKind::Searxng,
    )
    .await
    .map_err(|_| RuntimeError::EnvironmentUnavailable)?;
    let status = tokio::select! {
        result = child.wait() => result.map_err(|_| RuntimeError::EnvironmentUnavailable)?,
        _ = cancel.cancelled() => {
            crate::services::process_tree::terminate_tokio(
                &mut child,
                crate::services::process_tree::ProcessKind::Searxng,
            ).await;
            return Err(RuntimeError::Cancelled);
        }
    };
    if status.success() {
        Ok(())
    } else {
        Err(RuntimeError::EnvironmentUnavailable)
    }
}

fn venv_python(venv: &Path) -> PathBuf {
    if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    }
}
