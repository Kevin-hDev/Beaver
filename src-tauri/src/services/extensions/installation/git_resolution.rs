//! Pin moving branch/tag names before starting a replayable materialization.
use super::{install_signal::InstallSignal, source_validation::GitSource};
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub(super) fn pin(
    source: &GitSource,
    destination: &Path,
    signal: &impl InstallSignal,
    remaining: &mut Duration,
) -> Result<GitSource, git2::Error> {
    if source
        .reference
        .as_deref()
        .is_some_and(super::git_source::looks_like_full_commit)
    {
        return Ok(source.clone());
    }
    let directory = destination.join(".resolve-git");
    let started = Instant::now();
    let repository = git2::Repository::init_bare(&directory)?;
    let mut remote = repository.remote_anonymous(&source.clone_url)?;
    let connection = remote.connect_auth(
        git2::Direction::Fetch,
        Some(super::git_transport::callbacks(
            started + *remaining,
            signal.clone(),
        )?),
        None,
    )?;
    let heads = connection.list()?;
    if heads.len() > super::git_source::MAX_GIT_OBJECTS {
        return Err(git2::Error::from_str("remote references exceed budget"));
    }
    let reference = source.reference.as_deref().unwrap_or("HEAD");
    let names = [
        format!("{reference}^{{}}"),
        format!("refs/tags/{reference}^{{}}"),
        reference.to_owned(),
        format!("refs/heads/{reference}"),
        format!("refs/tags/{reference}"),
    ];
    let resolved = names.iter().find_map(|name| {
        heads
            .iter()
            .find(|head| head.name() == name)
            .map(|head| head.oid().to_string())
    });
    let tips: Vec<String> = heads
        .iter()
        .map(|head| head.oid().to_string())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    drop(connection);
    *remaining = remaining.saturating_sub(started.elapsed());
    signal
        .after_producer_stopped()
        .map_err(|_| git2::Error::from_str("resolution stopped"))?;
    let resolved = match resolved {
        Some(value) => value,
        None if super::git_source::looks_like_short_commit(reference) => {
            super::git_resolution_history::resolve(
                &repository,
                &mut remote,
                &tips,
                reference,
                signal,
                remaining,
            )?
        }
        None => return Err(git2::Error::from_str("remote reference unavailable")),
    };
    drop(remote);
    drop(repository);
    std::fs::remove_dir_all(directory)
        .map_err(|_| git2::Error::from_str("resolution cleanup failed"))?;
    Ok(GitSource {
        locator: format!("{}#{resolved}", source.clone_url),
        clone_url: source.clone_url.clone(),
        reference: Some(resolved),
    })
}

pub(super) fn remove_partial(checkout: &Path) -> Result<(), super::OperationFailure> {
    match std::fs::symlink_metadata(checkout) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            std::fs::remove_dir_all(checkout).map_err(|_| super::OperationFailure::StorageFailed)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(super::OperationFailure::StorageFailed),
    }
}
