use super::npm_runner::NpmRunner;
use super::source_validation::GitSource;
use super::{git_reference::checkout_reference, git_transport::fetch_options};
use git2::build::RepoBuilder;
use git2::Repository;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::install_signal::InstallSignal;

pub(super) const MAX_GIT_OBJECTS: usize = 100_000;
const MIN_COMMIT_PREFIX_CHARS: usize = 7;
const SHA1_CHARS: usize = 40;
const SHA256_CHARS: usize = 64;
const GIT_TIMEOUT: Duration = Duration::from_secs(300);

pub struct GitMaterialization {
    pub root: PathBuf,
    pub revision: String,
}

pub fn materialize(
    source: &GitSource,
    destination: &Path,
    npm: &NpmRunner,
    cancellation: &impl InstallSignal,
) -> Result<GitMaterialization, super::OperationFailure> {
    cancellation.phase(super::install_jobs::InstallPhase::Downloading)?;
    let mut remaining = GIT_TIMEOUT;
    let pinned = super::git_resolution::pin(source, destination, cancellation, &mut remaining)
        .map_err(|error| clone_failure(&error, GIT_TIMEOUT.saturating_sub(remaining)))?;
    cancellation.resolved_git(&pinned)?;
    let checkout = destination.join("repository");
    let repository = loop {
        if remaining.is_zero() {
            return Err(super::OperationFailure::GitTimeout);
        }
        let attempt_started = Instant::now();
        let result = clone_with_timeout(&pinned, &checkout, cancellation, remaining)
            .map_err(|error| clone_failure(&error, attempt_started.elapsed()));
        remaining = remaining.saturating_sub(attempt_started.elapsed());
        let continued = cancellation
            .after_producer_stopped()
            .map_err(super::OperationFailure::from)?;
        match result {
            Ok(repository) => break repository,
            Err(_) if continued => super::git_resolution::remove_partial(&checkout)?,
            Err(error) => return Err(error),
        }
    };
    let revision = repository
        .head()
        .and_then(|head| head.peel_to_commit())
        .map(|commit| commit.id().to_string())
        .map_err(|_| super::OperationFailure::GitDownloadFailed)?;
    cancellation.resolved_git(&super::source_validation::GitSource {
        locator: format!("{}#{revision}", pinned.clone_url),
        clone_url: pinned.clone_url,
        reference: Some(revision.clone()),
    })?;
    drop(repository);
    std::fs::remove_dir_all(checkout.join(".git"))
        .map_err(|_| super::OperationFailure::StorageFailed)?;
    super::managed_tree::measure_with_budget(destination, cancellation.storage_budget())?;
    if super::git_package::has_runtime_dependencies(&checkout)? {
        cancellation.phase(super::install_jobs::InstallPhase::Dependencies)?;
        npm.install_dependencies(&checkout, cancellation)?;
    }
    super::managed_tree::measure_with_budget(destination, cancellation.storage_budget())?;
    Ok(GitMaterialization {
        root: checkout,
        revision,
    })
}

#[cfg(test)]
pub(super) fn clone_repository(
    source: &GitSource,
    checkout: &Path,
    cancellation: &impl InstallSignal,
) -> Result<Repository, git2::Error> {
    clone_with_timeout(source, checkout, cancellation, GIT_TIMEOUT)
}

fn clone_with_timeout(
    source: &GitSource,
    checkout: &Path,
    cancellation: &impl InstallSignal,
    timeout: Duration,
) -> Result<Repository, git2::Error> {
    let deadline = Instant::now() + timeout;
    let fetch = fetch_options(
        deadline,
        should_use_shallow_clone(source),
        cancellation.clone(),
    )?;
    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch);
    builder.with_checkout(super::git_checkout::bounded(cancellation.clone()));
    let repository = builder.clone(&source.clone_url, checkout)?;
    if let Some(reference) = &source.reference {
        checkout_reference(&repository, reference, deadline, cancellation)?;
    }
    Ok(repository)
}

pub(super) fn should_use_shallow_clone(source: &GitSource) -> bool {
    !source.clone_url.starts_with("file://")
        && !source
            .reference
            .as_deref()
            .is_some_and(looks_like_full_commit)
}

pub(super) fn looks_like_full_commit(reference: &str) -> bool {
    matches!(reference.len(), SHA1_CHARS | SHA256_CHARS)
        && reference
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

pub(super) fn looks_like_short_commit(reference: &str) -> bool {
    (MIN_COMMIT_PREFIX_CHARS..SHA1_CHARS).contains(&reference.len())
        && reference
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

pub(super) fn clone_failure(error: &git2::Error, elapsed: Duration) -> super::OperationFailure {
    let network_deadline = matches!(
        error.class(),
        git2::ErrorClass::Net
            | git2::ErrorClass::Ssl
            | git2::ErrorClass::Ssh
            | git2::ErrorClass::Http
    ) && elapsed
        >= crate::services::git::network_policy::timeout_classification_threshold();
    let callback_deadline = error.class() == git2::ErrorClass::Callback && elapsed >= GIT_TIMEOUT;
    if error.code() == git2::ErrorCode::Timeout || network_deadline || callback_deadline {
        super::OperationFailure::GitTimeout
    } else {
        super::OperationFailure::GitDownloadFailed
    }
}
