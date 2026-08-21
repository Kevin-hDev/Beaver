use std::path::Path;

use super::runtime_error::RuntimeError;

pub(super) fn recover(current: &Path, staged: &Path, previous: &Path) -> Result<(), RuntimeError> {
    require_siblings(current, staged, previous)?;
    remove_if_present(staged)?;
    match (present(current)?, present(previous)?) {
        (true, true) => remove_if_present(previous),
        (false, true) => rename(previous, current),
        _ => Ok(()),
    }
}

pub(super) fn publish(current: &Path, staged: &Path, previous: &Path) -> Result<(), RuntimeError> {
    require_siblings(current, staged, previous)?;
    if !present(staged)? || present(previous)? {
        return Err(RuntimeError::WheelhouseUnavailable);
    }
    if present(current)? {
        rename(current, previous)?;
    }
    if let Err(error) = rename(staged, current) {
        if !present(current)? && present(previous)? {
            rename(previous, current)?;
        }
        return Err(error);
    }
    remove_if_present(previous)
}

fn require_siblings(current: &Path, staged: &Path, previous: &Path) -> Result<(), RuntimeError> {
    let parent = current
        .parent()
        .ok_or(RuntimeError::WheelhouseUnavailable)?;
    if staged.parent() == Some(parent)
        && previous.parent() == Some(parent)
        && current != staged
        && current != previous
        && staged != previous
    {
        Ok(())
    } else {
        Err(RuntimeError::WheelhouseUnavailable)
    }
}

fn present(path: &Path) -> Result<bool, RuntimeError> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() => Ok(true),
        Ok(_) => Err(RuntimeError::WheelhouseUnavailable),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(RuntimeError::WheelhouseUnavailable),
    }
}

fn remove_if_present(path: &Path) -> Result<(), RuntimeError> {
    if present(path)? {
        std::fs::remove_dir_all(path).map_err(|_| RuntimeError::WheelhouseUnavailable)?;
    }
    Ok(())
}

fn rename(from: &Path, to: &Path) -> Result<(), RuntimeError> {
    if !present(from)? || present(to)? {
        return Err(RuntimeError::WheelhouseUnavailable);
    }
    std::fs::rename(from, to).map_err(|_| RuntimeError::WheelhouseUnavailable)
}
