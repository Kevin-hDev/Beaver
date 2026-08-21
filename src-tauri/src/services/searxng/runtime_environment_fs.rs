use std::fs;
use std::path::{Path, PathBuf};

use super::runtime_error::RuntimeError;

pub(super) struct Layout {
    root: PathBuf,
    pub(super) current: PathBuf,
    pub(super) staged: PathBuf,
    pub(super) previous: PathBuf,
}

impl Layout {
    pub(super) fn production() -> Result<Self, RuntimeError> {
        let root = super::paths::sidecar_dir();
        fs::create_dir_all(&root).map_err(|_| RuntimeError::EnvironmentUnavailable)?;
        let layout = Self::at(&root)?;
        if layout.current != super::paths::venv_dir()
            || layout.staged != super::paths::staged_venv_dir()
            || layout.previous != super::paths::previous_venv_dir()
        {
            return Err(RuntimeError::EnvironmentUnavailable);
        }
        Ok(layout)
    }

    pub(super) fn at(root: &Path) -> Result<Self, RuntimeError> {
        let metadata =
            fs::symlink_metadata(root).map_err(|_| RuntimeError::EnvironmentUnavailable)?;
        if !metadata.file_type().is_dir() {
            return Err(RuntimeError::EnvironmentUnavailable);
        }
        Ok(Self {
            root: root.to_path_buf(),
            // Les tests changent uniquement la racine ; les suffixes restent
            // ceux de l'autorité de chemins utilisée en production.
            current: root.join(super::paths::VENV_NAME),
            staged: root.join(super::paths::STAGED_VENV_NAME),
            previous: root.join(super::paths::PREVIOUS_VENV_NAME),
        })
    }
}

pub(super) fn recover(layout: &Layout) -> Result<(), RuntimeError> {
    super::generational_publication::recover(
        paths(layout),
        super::generational_publication::RecoveryPolicy::RollBackUnconfirmed,
        RuntimeError::EnvironmentUnavailable,
    )
}

pub(super) fn prepare_staging(layout: &Layout) -> Result<(), RuntimeError> {
    super::generational_publication::prepare_staging(
        paths(layout),
        RuntimeError::EnvironmentUnavailable,
    )
}

pub(super) fn publish(layout: &Layout) -> Result<(), RuntimeError> {
    super::generational_publication::publish(
        paths(layout),
        super::generational_publication::RecoveryPolicy::RollBackUnconfirmed,
        RuntimeError::EnvironmentUnavailable,
    )
}

pub(super) fn paths(layout: &Layout) -> super::generational_publication::Paths<'_> {
    super::generational_publication::Paths {
        current: &layout.current,
        staged: &layout.staged,
        previous: &layout.previous,
    }
}

pub(super) fn present_dir(path: &Path) -> Result<bool, RuntimeError> {
    super::generational_publication::present(path, RuntimeError::EnvironmentUnavailable)
}

pub(super) fn regular_executable(path: &Path) -> Result<bool, RuntimeError> {
    // Un venv CPython expose `bin/python` comme lien vers l'interpréteur réel.
    // On suit uniquement ce lien exécutable ; les dossiers restent contrôlés sans suivi.
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => executable_metadata(&metadata),
        Ok(_) => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(RuntimeError::EnvironmentUnavailable),
    }
}

#[cfg(unix)]
fn executable_metadata(metadata: &fs::Metadata) -> Result<bool, RuntimeError> {
    use std::os::unix::fs::PermissionsExt;

    Ok(metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn executable_metadata(_: &fs::Metadata) -> Result<bool, RuntimeError> {
    Ok(true)
}

pub(super) fn remove_dir(layout: &Layout, path: &Path) -> Result<(), RuntimeError> {
    require_layout_path(layout, path)?;
    super::generational_publication::remove_if_present(path, RuntimeError::EnvironmentUnavailable)
}

fn require_layout_path(layout: &Layout, path: &Path) -> Result<(), RuntimeError> {
    let allowed = [&layout.current, &layout.staged, &layout.previous];
    if allowed.into_iter().any(|candidate| candidate == path) && path.parent() == Some(&layout.root)
    {
        Ok(())
    } else {
        Err(RuntimeError::EnvironmentUnavailable)
    }
}
