use std::path::{Path, PathBuf};
use tokio::time::Instant;

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
        let deadline = Instant::now() + super::runtime_command::RUNTIME_INSTALL_TIMEOUT;
        Self::ensure_with_layout_deadline(
            source, wheelhouse, python, cancel, deadline, layout, publish,
        )
        .await
    }

    pub(super) async fn ensure_with_layout_deadline<Publish>(
        source: &Path,
        wheelhouse: &Wheelhouse,
        python: &PythonRuntime,
        cancel: &ServiceWorkCancellation,
        deadline: Instant,
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
        execute(
            &mut command,
            super::runtime_command::RuntimeStage::CreateVenv,
            deadline,
            cancel,
        )
        .await?;
        let staged_python = python.with_program(venv_python(&layout.staged));
        install(&staged_python, wheelhouse, source, deadline, cancel).await?;
        smoke(&staged_python, deadline, cancel).await?;
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
}

async fn install(
    python: &PythonRuntime,
    wheelhouse: &Wheelhouse,
    source: &Path,
    deadline: Instant,
    cancel: &ServiceWorkCancellation,
) -> Result<(), RuntimeError> {
    let mut tools = python.command();
    tools
        .args(["-m", "pip", "install", "--no-index", "--find-links"])
        .arg(&wheelhouse.path)
        .args(["setuptools", "wheel"]);
    execute(
        &mut tools,
        super::runtime_command::RuntimeStage::InstallBuildTools,
        deadline,
        cancel,
    )
    .await?;

    let mut requirements = python.command();
    requirements
        .args(["-m", "pip", "install", "--no-index", "--find-links"])
        .arg(&wheelhouse.path)
        .args(["-r"])
        .arg(source.join("requirements.txt"));
    execute(
        &mut requirements,
        super::runtime_command::RuntimeStage::InstallRequirements,
        deadline,
        cancel,
    )
    .await
}

async fn smoke(
    python: &PythonRuntime,
    deadline: Instant,
    cancel: &ServiceWorkCancellation,
) -> Result<(), RuntimeError> {
    let mut command = python.command();
    command.args(["-c", "import lxml, markupsafe, msgspec, yaml, searx.webapp"]);
    execute(
        &mut command,
        super::runtime_command::RuntimeStage::ValidateImports,
        deadline,
        cancel,
    )
    .await
}

async fn execute(
    command: &mut tokio::process::Command,
    stage: super::runtime_command::RuntimeStage,
    deadline: Instant,
    cancel: &ServiceWorkCancellation,
) -> Result<(), RuntimeError> {
    super::runtime_command::run_runtime_command(command, stage, deadline, cancel)
        .await
        .map_err(|error| {
            ::log::warn!(
                "[searxng] runtime stage={} category={}",
                error.stage().as_str(),
                error.category()
            );
            RuntimeError::EnvironmentUnavailable
        })
}

fn venv_python(venv: &Path) -> PathBuf {
    if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    }
}
