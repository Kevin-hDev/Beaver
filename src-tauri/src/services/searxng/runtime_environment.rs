use std::path::{Path, PathBuf};

use crate::services::work_registry::ServiceWorkCancellation;

use super::python_runtime::PythonRuntime;
use super::runtime_error::RuntimeError;
use super::wheels::Wheelhouse;

pub(super) struct RuntimeEnvironment;

impl RuntimeEnvironment {
    pub(super) async fn ensure(
        source: &Path,
        wheelhouse: &Wheelhouse,
        python: &PythonRuntime,
        cancel: &ServiceWorkCancellation,
    ) -> Result<PathBuf, RuntimeError> {
        let layout = super::runtime_environment_fs::Layout::production()?;
        Self::ensure_with_layout(
            source,
            wheelhouse,
            python,
            cancel,
            &layout,
            super::runtime_environment_fs::publish,
        )
        .await
    }

    pub(super) async fn ensure_with_layout<Publish>(
        source: &Path,
        wheelhouse: &Wheelhouse,
        python: &PythonRuntime,
        cancel: &ServiceWorkCancellation,
        layout: &super::runtime_environment_fs::Layout,
        publish: Publish,
    ) -> Result<PathBuf, RuntimeError>
    where
        Publish: FnOnce(&super::runtime_environment_fs::Layout) -> Result<(), RuntimeError>,
    {
        super::runtime_environment_fs::recover(layout)?;
        let source_hash = super::runtime_receipt::source_hash(source)?;
        let current_python = venv_python(&layout.current);
        if super::runtime_receipt::reusable(layout, &wheelhouse.manifest, &source_hash)?
            && super::runtime_environment_fs::regular_executable(&current_python)?
        {
            return Ok(current_python);
        }
        super::runtime_environment_fs::prepare_staging(layout)?;
        let mut command = python.command();
        command.args(["-m", "venv"]).arg(&layout.staged);
        super::runtime::run(command, cancel).await?;
        let staged_python = python.with_program(venv_python(&layout.staged));
        install(&staged_python, wheelhouse, source, cancel).await?;
        smoke(&staged_python, cancel).await?;
        if cancel.is_cancelled() {
            return Err(RuntimeError::Cancelled);
        }
        super::runtime_receipt::write_receipt(layout, &wheelhouse.manifest, &source_hash)?;
        publish(layout)?;
        Ok(venv_python(&layout.current))
    }

    pub(super) fn mark_started() -> Result<(), RuntimeError> {
        let layout = super::runtime_environment_fs::Layout::production()?;
        if super::runtime_environment_fs::present_dir(&layout.previous)? {
            super::runtime_environment_fs::remove_dir(&layout, &layout.previous)?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn recover_at(root: &Path) -> Result<(), RuntimeError> {
        super::runtime_environment_fs::recover(&super::runtime_environment_fs::Layout::at(root)?)
    }

    #[cfg(test)]
    pub(super) fn reusable_at(
        root: &Path,
        manifest: &super::runtime_manifest::RuntimeManifest,
        source_hash: &str,
    ) -> Result<bool, RuntimeError> {
        super::runtime_receipt::reusable(
            &super::runtime_environment_fs::Layout::at(root)?,
            manifest,
            source_hash,
        )
    }

    #[cfg(test)]
    pub(super) fn staging_target_at(root: &Path) -> Result<PathBuf, RuntimeError> {
        Ok(super::runtime_environment_fs::Layout::at(root)?.staged)
    }

    #[cfg(test)]
    pub(super) fn prepare_staging_at(root: &Path) -> Result<(), RuntimeError> {
        super::runtime_environment_fs::prepare_staging(&super::runtime_environment_fs::Layout::at(
            root,
        )?)
    }

    #[cfg(test)]
    pub(super) fn executable_runtime_at(root: &Path) -> Result<bool, RuntimeError> {
        let layout = super::runtime_environment_fs::Layout::at(root)?;
        super::runtime_environment_fs::regular_executable(&venv_python(&layout.current))
    }

    #[cfg(test)]
    pub(super) fn publish_at<F>(root: &Path, publish_next: F) -> Result<(), RuntimeError>
    where
        F: FnOnce(&Path, &Path) -> Result<(), RuntimeError>,
    {
        super::runtime_environment_fs::publish_with(
            &super::runtime_environment_fs::Layout::at(root)?,
            publish_next,
        )
    }

    #[cfg(test)]
    pub(super) fn mark_started_at(root: &Path) -> Result<(), RuntimeError> {
        let layout = super::runtime_environment_fs::Layout::at(root)?;
        if super::runtime_environment_fs::present_dir(&layout.previous)? {
            super::runtime_environment_fs::remove_dir(&layout, &layout.previous)?;
        }
        Ok(())
    }
}

async fn install(
    python: &PythonRuntime,
    wheelhouse: &Wheelhouse,
    source: &Path,
    cancel: &ServiceWorkCancellation,
) -> Result<(), RuntimeError> {
    let mut tools = python.command();
    tools
        .args(["-m", "pip", "install", "--no-index", "--find-links"])
        .arg(&wheelhouse.path)
        .args(["setuptools", "wheel"]);
    super::runtime::run(tools, cancel).await?;

    let mut requirements = python.command();
    requirements
        .args(["-m", "pip", "install", "--no-index", "--find-links"])
        .arg(&wheelhouse.path)
        .args(["-r"])
        .arg(source.join("requirements.txt"));
    super::runtime::run(requirements, cancel).await
}

async fn smoke(
    python: &PythonRuntime,
    cancel: &ServiceWorkCancellation,
) -> Result<(), RuntimeError> {
    let mut command = python.command();
    command.args(["-c", "import lxml, markupsafe, msgspec, yaml, searx.webapp"]);
    super::runtime::run(command, cancel).await
}

fn venv_python(venv: &Path) -> PathBuf {
    if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    }
}
