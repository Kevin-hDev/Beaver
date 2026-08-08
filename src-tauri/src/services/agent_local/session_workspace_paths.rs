use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub(super) fn relative_workspace_path(base: &Path, work: &Path) -> Result<PathBuf, String> {
    relative_workspace_path_with(base, work, |path| std::fs::canonicalize(path))
}

pub(super) fn relative_workspace_path_with<F>(
    base: &Path,
    work: &Path,
    canonicalize: F,
) -> Result<PathBuf, String>
where
    F: Fn(&Path) -> std::io::Result<PathBuf>,
{
    if let Ok(relative) = dunce::simplified(work).strip_prefix(dunce::simplified(base)) {
        return Ok(relative.to_path_buf());
    }
    canonical_path_and_relative_with(base, work, canonicalize).map(|(_, relative)| relative)
}

pub(super) fn reject_symlinks(base: &Path, target: &Path) -> Result<(), String> {
    let mut current = target;
    loop {
        match std::fs::symlink_metadata(current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(workspace_error());
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(_) => return Err(workspace_error()),
        }
        if paths_identify_same_directory(base, current)? {
            return Ok(());
        }
        current = current.parent().ok_or_else(workspace_error)?;
    }
}

pub(super) fn validate_created_path(base: &Path, path: &Path) -> Result<(), String> {
    let (canonical_path, _) =
        canonical_path_and_relative_with(base, path, |path| std::fs::canonicalize(path))?;
    if canonical_path.is_dir() {
        Ok(())
    } else {
        Err(workspace_error())
    }
}

fn canonical_path_and_relative_with<F>(
    base: &Path,
    path: &Path,
    canonicalize: F,
) -> Result<(PathBuf, PathBuf), String>
where
    F: Fn(&Path) -> std::io::Result<PathBuf>,
{
    let canonical_base = canonicalize(base).map_err(|_| workspace_error())?;
    let canonical_path = canonicalize(path).map_err(|_| workspace_error())?;
    let relative = canonical_path
        .strip_prefix(canonical_base)
        .map(Path::to_path_buf)
        .map_err(|_| workspace_error())?;
    Ok((canonical_path, relative))
}

fn paths_identify_same_directory(base: &Path, candidate: &Path) -> Result<bool, String> {
    if dunce::simplified(base) == dunce::simplified(candidate) {
        return Ok(true);
    }
    let base_exists = path_exists(base)?;
    let candidate_exists = path_exists(candidate)?;
    if !base_exists || !candidate_exists {
        return Ok(false);
    }
    let canonical_base = std::fs::canonicalize(base).map_err(|_| workspace_error())?;
    let canonical_candidate = std::fs::canonicalize(candidate).map_err(|_| workspace_error())?;
    Ok(canonical_base == canonical_candidate)
}

fn path_exists(path: &Path) -> Result<bool, String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(_) => Err(workspace_error()),
    }
}

pub(super) fn workspace_error() -> String {
    "Espace de travail indisponible.".to_string()
}
