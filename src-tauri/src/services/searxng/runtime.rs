use crate::services::work_registry::ServiceWorkCancellation;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;

const STAMP: &str = ".runtime.stamp";

pub async fn ensure_runtime(
    source: &Path,
    cancel: &ServiceWorkCancellation,
) -> Result<PathBuf, String> {
    validate_source(source)?;
    let venv = super::paths::venv_dir();
    let python = venv_python(&venv);
    if !python.exists() {
        let base_python = find_python()?;
        let mut command = Command::new(base_python);
        command.args(["-m", "venv"]).arg(&venv);
        run(command, cancel).await?;
    }

    let stamp_path = venv.join(STAMP);
    let stamp = source_stamp(source)?;
    let installed = std::fs::read_to_string(&stamp_path).unwrap_or_default();
    if installed != stamp {
        install_build_tools(&python, source, cancel).await?;
        install_requirements(&python, source, cancel).await?;
        if cancel.is_cancelled() {
            return Err("SearXNG: arrêt en cours".to_string());
        }
        std::fs::write(stamp_path, stamp)
            .map_err(|_| "SearXNG: validation runtime impossible".to_string())?;
    }
    Ok(python)
}

fn validate_source(source: &Path) -> Result<(), String> {
    let required = ["setup.py", "requirements.txt", "LICENSE", "searx/webapp.py"];
    if required.iter().all(|file| source.join(file).exists()) {
        Ok(())
    } else {
        Err("SearXNG: bundle incomplet".to_string())
    }
}

fn source_stamp(source: &Path) -> Result<String, String> {
    let mut hasher = Sha256::new();
    for file in ["setup.py", "requirements.txt", "searx/version.py"] {
        let body = std::fs::read(source.join(file))
            .map_err(|_| "SearXNG: bundle incomplet".to_string())?;
        hasher.update(body);
    }
    Ok(hex::encode(hasher.finalize()))
}

async fn install_build_tools(
    python: &Path,
    source: &Path,
    cancel: &ServiceWorkCancellation,
) -> Result<(), String> {
    let mut command = Command::new(python);
    if let Some(wheels) = wheelhouse_for_source(source) {
        command
            .args(["-m", "pip", "install", "--no-index", "--find-links"])
            .arg(wheels)
            .args(["setuptools", "wheel"]);
    } else {
        command.args([
            "-m",
            "pip",
            "install",
            "--upgrade",
            "pip",
            "setuptools",
            "wheel",
        ]);
    }
    run(command, cancel).await
}

async fn install_requirements(
    python: &Path,
    source: &Path,
    cancel: &ServiceWorkCancellation,
) -> Result<(), String> {
    let requirements = source.join("requirements.txt");
    let mut command = Command::new(python);
    if let Some(wheels) = wheelhouse_for_source(source) {
        command
            .args(["-m", "pip", "install", "--no-index", "--find-links"])
            .arg(wheels)
            .arg("-r")
            .arg(requirements);
    } else {
        command
            .args(["-m", "pip", "install", "-r"])
            .arg(requirements);
    }
    run(command, cancel).await
}

async fn run(mut command: Command, cancel: &ServiceWorkCancellation) -> Result<(), String> {
    crate::services::process_tree::configure_tokio(&mut command);
    command
        .env("PIP_DISABLE_PIP_VERSION_CHECK", "1")
        .env("PIP_NO_INPUT", "1")
        .env("PYTHONUNBUFFERED", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = command
        .spawn()
        .map_err(|_| "SearXNG: runtime indisponible".to_string())?;
    let status = tokio::select! {
        result = child.wait() => result.map_err(|_| "SearXNG: runtime indisponible".to_string())?,
        _ = cancel.cancelled() => {
            crate::services::process_tree::terminate_tokio(
                &mut child,
                crate::services::process_tree::ProcessKind::Searxng,
            ).await;
            return Err("SearXNG: arrêt en cours".to_string());
        }
    };
    if status.success() {
        Ok(())
    } else {
        Err("SearXNG: installation runtime échouée".to_string())
    }
}

fn wheelhouse_for_source(source: &Path) -> Option<PathBuf> {
    super::wheels::for_source(source)
}

fn find_python() -> Result<PathBuf, String> {
    for candidate in [
        "python3.13",
        "python3.12",
        "python3.11",
        "python3.10",
        "python3",
        "python",
    ] {
        if let Ok(path) = which::which(candidate) {
            return Ok(path);
        }
    }
    Err("SearXNG: runtime Python introuvable".to_string())
}

fn venv_python(venv: &Path) -> PathBuf {
    if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    }
}
