use super::npm_runner::NpmRunner;
use super::source_validation::GitSource;
use git2::build::RepoBuilder;
use git2::{AutotagOption, FetchOptions, RemoteCallbacks, Repository};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const MAX_TRANSFER_BYTES: usize = 512 * 1024 * 1024;
const MAX_GIT_OBJECTS: usize = 100_000;
const MIN_COMMIT_PREFIX_CHARS: usize = 7;
const MAX_COMMIT_CHARS: usize = 64;
const GIT_TIMEOUT: Duration = Duration::from_secs(300);

pub struct GitMaterialization {
    pub root: PathBuf,
    pub revision: String,
}

pub fn materialize(
    source: &GitSource,
    destination: &Path,
    npm: &NpmRunner,
) -> Result<GitMaterialization, super::OperationFailure> {
    let started = Instant::now();
    let checkout = destination.join("repository");
    let repository = clone_repository(source, &checkout)
        .map_err(|error| clone_failure(&error, started.elapsed()))?;
    let revision = repository
        .head()
        .and_then(|head| head.peel_to_commit())
        .map(|commit| commit.id().to_string())
        .map_err(|_| super::OperationFailure::GitDownloadFailed)?;
    drop(repository);
    std::fs::remove_dir_all(checkout.join(".git"))
        .map_err(|_| super::OperationFailure::StorageFailed)?;
    super::managed_tree::validate(destination)?;
    if super::git_package::has_runtime_dependencies(&checkout)? {
        npm.install_dependencies(&checkout)?;
    }
    super::managed_tree::validate(destination)?;
    Ok(GitMaterialization {
        root: checkout,
        revision,
    })
}

pub(super) fn clone_repository(
    source: &GitSource,
    checkout: &Path,
) -> Result<Repository, git2::Error> {
    let deadline = Instant::now() + GIT_TIMEOUT;
    let fetch = fetch_options(deadline, should_use_shallow_clone(source))?;
    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch);
    builder.with_checkout(super::git_checkout::bounded());
    let repository = builder.clone(&source.clone_url, checkout)?;
    if let Some(reference) = &source.reference {
        checkout_reference(&repository, source, reference, deadline)?;
    }
    Ok(repository)
}

fn fetch_options(deadline: Instant, shallow: bool) -> Result<FetchOptions<'static>, git2::Error> {
    let config = git2::Config::open_default()
        .map_err(|_| git2::Error::from_str("git configuration unavailable"))?;
    let mut credentials =
        crate::services::git::remote_credentials::CredentialProvider::new(config, None);
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |url, username, allowed| {
        if Instant::now() >= deadline {
            return Err(git2::Error::new(
                git2::ErrorCode::Timeout,
                git2::ErrorClass::Net,
                "extension clone expired",
            ));
        }
        credentials.credentials(url, username, allowed)
    });
    callbacks.transfer_progress(move |progress| {
        Instant::now() < deadline
            && progress.received_bytes() <= MAX_TRANSFER_BYTES
            && progress.total_objects() <= MAX_GIT_OBJECTS
    });
    let mut fetch = FetchOptions::new();
    if shallow {
        fetch.depth(1);
    }
    fetch
        .download_tags(AutotagOption::Auto)
        .remote_callbacks(callbacks);
    Ok(fetch)
}

fn checkout_reference(
    repository: &Repository,
    source: &GitSource,
    reference: &str,
    deadline: Instant,
) -> Result<(), git2::Error> {
    if let Some(commit) = resolve_commit(repository, reference) {
        repository.checkout_tree(
            commit.as_object(),
            Some(&mut super::git_checkout::bounded()),
        )?;
        return repository.set_head_detached(commit.id());
    }
    let mut remote = repository.find_remote("origin")?;
    let mut fetch = fetch_options(deadline, should_use_shallow_clone(source))?;
    remote.fetch(&[reference], Some(&mut fetch), None)?;
    let commit = resolve_commit(repository, reference)
        .ok_or_else(|| git2::Error::from_str("git reference unavailable"))?;
    repository.checkout_tree(
        commit.as_object(),
        Some(&mut super::git_checkout::bounded()),
    )?;
    repository.set_head_detached(commit.id())
}

fn resolve_commit<'a>(repository: &'a Repository, reference: &str) -> Option<git2::Commit<'a>> {
    let short = reference
        .strip_prefix("refs/heads/")
        .or_else(|| reference.strip_prefix("refs/tags/"))
        .unwrap_or(reference);
    [
        reference.to_string(),
        format!("refs/remotes/origin/{short}"),
        format!("refs/tags/{short}"),
    ]
    .iter()
    .find_map(|candidate| {
        repository
            .revparse_single(candidate)
            .ok()
            .and_then(|object| object.peel_to_commit().ok())
    })
}

pub(super) fn should_use_shallow_clone(source: &GitSource) -> bool {
    !source.clone_url.starts_with("file://")
        && !source.reference.as_deref().is_some_and(looks_like_commit)
}

fn looks_like_commit(reference: &str) -> bool {
    (MIN_COMMIT_PREFIX_CHARS..=MAX_COMMIT_CHARS).contains(&reference.len())
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
