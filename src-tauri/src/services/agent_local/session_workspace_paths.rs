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
    canonical_contained_path_and_relative_with(base, work, canonicalize)
        .map(|(_, relative)| relative)
}

pub(super) fn reject_symlinks(base: &Path, target: &Path) -> Result<(), String> {
    let canonical_base = if dunce::simplified(target)
        .strip_prefix(dunce::simplified(base))
        .is_err()
    {
        canonicalize_optional(base)?
    } else {
        None
    };
    let mut current = target;
    loop {
        let current_exists = match std::fs::symlink_metadata(current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(workspace_error());
            }
            Ok(_) => true,
            Err(error) if error.kind() == ErrorKind::NotFound => false,
            Err(_) => return Err(workspace_error()),
        };
        if paths_identify_same_directory(base, canonical_base.as_deref(), current, current_exists)?
        {
            return Ok(());
        }
        current = current.parent().ok_or_else(workspace_error)?;
    }
}

pub(super) fn validate_created_path(base: &Path, path: &Path) -> Result<(), String> {
    let (canonical_path, _) =
        canonical_contained_path_and_relative_with(base, path, |path| std::fs::canonicalize(path))?;
    if canonical_path.is_dir() {
        Ok(())
    } else {
        Err(workspace_error())
    }
}

fn canonical_contained_path_and_relative_with<F>(
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

fn paths_identify_same_directory(
    base: &Path,
    canonical_base: Option<&Path>,
    candidate: &Path,
    candidate_exists: bool,
) -> Result<bool, String> {
    if dunce::simplified(base) == dunce::simplified(candidate) {
        return Ok(true);
    }
    let Some(canonical_base) = canonical_base.filter(|_| candidate_exists) else {
        return Ok(false);
    };
    let canonical_candidate = std::fs::canonicalize(candidate).map_err(|_| workspace_error())?;
    Ok(canonical_base == canonical_candidate.as_path())
}

fn canonicalize_optional(path: &Path) -> Result<Option<PathBuf>, String> {
    match std::fs::canonicalize(path) {
        Ok(path) => Ok(Some(path)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(_) => Err(workspace_error()),
    }
}

pub(super) fn workspace_error() -> String {
    "Espace de travail indisponible.".to_string()
}
