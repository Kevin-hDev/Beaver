use super::manifest::runtime_id;
use std::path::{Path, PathBuf};

const REQUIREMENTS_STAMP: &str = ".requirements.stamp";

#[derive(Debug)]
pub(super) struct RuntimePaths {
    pub(super) live: PathBuf,
    pub(super) staging: PathBuf,
    pub(super) backup: PathBuf,
}

impl RuntimePaths {
    pub(super) fn python_in(&self, dir: &Path) -> PathBuf {
        if cfg!(windows) {
            dir.join("Scripts").join("python.exe")
        } else {
            dir.join("bin").join("python")
        }
    }
}

pub(super) fn runtime_paths(sidecar_dir: &Path, family_id: &str) -> Result<RuntimePaths, String> {
    let runtime = runtime_id(family_id)?;
    let root = sidecar_dir.join(".venvs");
    Ok(RuntimePaths {
        live: root.join(runtime),
        staging: root.join(format!(".{runtime}.staging")),
        backup: root.join(format!(".{runtime}.backup")),
    })
}

pub(super) fn runtime_ready_at(dir: &Path, expected: &str) -> bool {
    let python = if cfg!(windows) {
        dir.join("Scripts").join("python.exe")
    } else {
        dir.join("bin").join("python")
    };
    if !python.exists() {
        return false;
    }
    std::fs::read_to_string(dir.join(REQUIREMENTS_STAMP))
        .is_ok_and(|installed| installed == expected)
}

pub(super) fn write_stamp(dir: &Path, requirements: &str) -> Result<(), String> {
    std::fs::write(dir.join(REQUIREMENTS_STAMP), requirements)
        .map_err(|_| "Validation du runtime Forecast impossible".to_string())
}

pub(super) fn commit_staged_runtime(paths: &RuntimePaths) -> Result<(), String> {
    if !paths.staging.exists() {
        return Err("Préparation du runtime Forecast impossible".to_string());
    }
    remove_if_exists(&paths.backup)?;
    let had_live = paths.live.exists();
    if had_live {
        std::fs::rename(&paths.live, &paths.backup)
            .map_err(|_| "Finalisation du runtime Forecast impossible".to_string())?;
    }
    if std::fs::rename(&paths.staging, &paths.live).is_err() {
        if had_live {
            let _ = std::fs::rename(&paths.backup, &paths.live);
        }
        return Err("Finalisation du runtime Forecast impossible".to_string());
    }
    let _ = remove_if_exists(&paths.backup);
    Ok(())
}

pub(super) fn remove_runtime_paths(paths: &RuntimePaths) -> Result<(), String> {
    remove_if_exists(&paths.staging)?;
    remove_if_exists(&paths.backup)?;
    remove_if_exists(&paths.live)
}

pub(super) fn remove_if_exists(path: &Path) -> Result<(), String> {
    if path.exists() {
        std::fs::remove_dir_all(path)
            .map_err(|_| "Suppression du runtime Forecast impossible".to_string())?;
    }
    Ok(())
}
