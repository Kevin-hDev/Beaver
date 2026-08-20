use crate::services::work_registry::ServiceWorkCancellation;
use std::path::{Path, PathBuf};

use super::runtime_error::RuntimeError;

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
