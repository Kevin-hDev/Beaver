use std::path::Path;

use super::runtime_error::RuntimeError;

#[derive(Clone, Copy)]
pub(super) enum RecoveryPolicy {
    /// The previous generation remains authoritative until a separate readiness gate succeeds.
    RollBackUnconfirmed,
    /// Publication itself validates the generation, so a present current generation wins.
    CommitImmediately,
}

pub(super) struct Paths<'a> {
    pub(super) current: &'a Path,
    pub(super) staged: &'a Path,
    pub(super) previous: &'a Path,
}

pub(super) fn recover(
    paths: Paths<'_>,
    policy: RecoveryPolicy,
    failure: RuntimeError,
) -> Result<(), RuntimeError> {
    require_siblings(&paths, failure)?;
    remove_if_present(paths.staged, failure)?;
    if present(paths.current, failure)? && present(paths.previous, failure)? {
        match policy {
            RecoveryPolicy::RollBackUnconfirmed => remove_if_present(paths.current, failure)?,
            RecoveryPolicy::CommitImmediately => remove_if_present(paths.previous, failure)?,
        }
    }
    if !present(paths.current, failure)? && present(paths.previous, failure)? {
        rename(paths.previous, paths.current, failure)?;
    }
    Ok(())
}

pub(super) fn prepare_staging(paths: Paths<'_>, failure: RuntimeError) -> Result<(), RuntimeError> {
    require_siblings(&paths, failure)?;
    if present(paths.staged, failure)? {
        return Err(failure);
    }
    std::fs::create_dir(paths.staged).map_err(|_| failure)
}

pub(super) fn publish(
    paths: Paths<'_>,
    policy: RecoveryPolicy,
    failure: RuntimeError,
) -> Result<(), RuntimeError> {
    publish_with(paths, policy, failure, |from, to| rename(from, to, failure))
}

pub(super) fn publish_with<F>(
    paths: Paths<'_>,
    policy: RecoveryPolicy,
    failure: RuntimeError,
    publish_next: F,
) -> Result<(), RuntimeError>
where
    F: FnOnce(&Path, &Path) -> Result<(), RuntimeError>,
{
    require_siblings(&paths, failure)?;
    if !present(paths.staged, failure)? || present(paths.previous, failure)? {
        return Err(failure);
    }
    if present(paths.current, failure)? {
        rename(paths.current, paths.previous, failure)?;
    }
    if let Err(error) = publish_next(paths.staged, paths.current) {
        if !present(paths.current, failure)? && present(paths.previous, failure)? {
            rename(paths.previous, paths.current, failure)?;
        }
        return Err(error);
    }
    if matches!(policy, RecoveryPolicy::CommitImmediately) {
        remove_if_present(paths.previous, failure)?;
    }
    Ok(())
}

fn require_siblings(paths: &Paths<'_>, failure: RuntimeError) -> Result<(), RuntimeError> {
    let parent = paths.current.parent().ok_or(failure)?;
    if paths.staged.parent() == Some(parent)
        && paths.previous.parent() == Some(parent)
        && paths.current != paths.staged
        && paths.current != paths.previous
        && paths.staged != paths.previous
    {
        Ok(())
    } else {
        Err(failure)
    }
}

pub(super) fn present(path: &Path, failure: RuntimeError) -> Result<bool, RuntimeError> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() => Ok(true),
        Ok(_) => Err(failure),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(failure),
    }
}

pub(super) fn remove_if_present(path: &Path, failure: RuntimeError) -> Result<(), RuntimeError> {
    if present(path, failure)? {
        std::fs::remove_dir_all(path).map_err(|_| failure)?;
    }
    Ok(())
}

fn rename(from: &Path, to: &Path, failure: RuntimeError) -> Result<(), RuntimeError> {
    if !present(from, failure)? || present(to, failure)? {
        return Err(failure);
    }
    std::fs::rename(from, to).map_err(|_| failure)
}
