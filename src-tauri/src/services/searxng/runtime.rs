use crate::services::work_registry::ServiceWorkCancellation;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

use super::runtime_error::RuntimeError;

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;

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
    let wheelhouse =
        super::wheels::for_source(source)?.ok_or(RuntimeError::WheelhouseUnavailable)?;
    let base_python = super::python_runtime::PythonRuntime::resolve(&wheelhouse.manifest).await?;
    super::runtime_environment::RuntimeEnvironment::ensure(
        source,
        &wheelhouse,
        &base_python,
        cancel,
    )
    .await
}

pub(super) async fn run(
    mut command: Command,
    cancel: &ServiceWorkCancellation,
) -> Result<(), RuntimeError> {
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
