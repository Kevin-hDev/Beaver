use super::npm_runner::NpmRunner;
use super::source_validation::GitSource;
use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{AutotagOption, CheckoutNotificationType, FetchOptions, RemoteCallbacks, Repository};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

const MAX_TRANSFER_BYTES: usize = 512 * 1024 * 1024;
const MAX_GIT_OBJECTS: usize = 100_000;
const GIT_TIMEOUT: Duration = Duration::from_secs(300);

pub struct GitMaterialization {
    pub root: PathBuf,
    pub revision: String,
}

pub fn materialize(
    source: &GitSource,
    destination: &Path,
    npm: &NpmRunner,
) -> Result<GitMaterialization, String> {
    let checkout = destination.join("repository");
    let repository = clone_repository(source, &checkout).map_err(|error| {
        if error.code() == git2::ErrorCode::Timeout {
            "Téléchargement Git expiré.".to_string()
        } else {
            "Téléchargement Git impossible.".to_string()
        }
    })?;
    let revision = repository
        .head()
        .and_then(|head| head.peel_to_commit())
        .map(|commit| commit.id().to_string())
        .map_err(|_| "Révision Git introuvable.".to_string())?;
    drop(repository);
    std::fs::remove_dir_all(checkout.join(".git"))
        .map_err(|_| "Métadonnées Git impossibles à nettoyer.".to_string())?;
    super::managed_tree::validate(destination)?;
    if has_runtime_dependencies(&checkout)? {
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
    let fetch = fetch_options(source, deadline)?;
    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch);
    builder.with_checkout(bounded_checkout());
    let repository = builder.clone(&source.clone_url, checkout)?;
    if let Some(reference) = &source.reference {
        checkout_reference(&repository, source, reference, deadline)?;
    }
    Ok(repository)
}

fn fetch_options(
    source: &GitSource,
    deadline: Instant,
) -> Result<FetchOptions<'static>, git2::Error> {
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
    if !cfg!(test) || !source.clone_url.starts_with("file://") {
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
    let mut remote = repository.find_remote("origin")?;
    let mut fetch = fetch_options(source, deadline)?;
    remote.fetch(&[reference], Some(&mut fetch), None)?;
    let short = reference
        .strip_prefix("refs/heads/")
        .or_else(|| reference.strip_prefix("refs/tags/"))
        .unwrap_or(reference);
    let candidates = [
        reference.to_string(),
        format!("refs/remotes/origin/{short}"),
        format!("refs/tags/{short}"),
    ];
    let commit = candidates
        .iter()
        .find_map(|candidate| {
            repository
                .revparse_single(candidate)
                .ok()
                .and_then(|object| object.peel_to_commit().ok())
        })
        .ok_or_else(|| git2::Error::from_str("git reference unavailable"))?;
    repository.checkout_tree(commit.as_object(), Some(&mut bounded_checkout()))?;
    repository.set_head_detached(commit.id())
}

fn bounded_checkout() -> CheckoutBuilder<'static> {
    let mut checkout = CheckoutBuilder::new();
    let mut paths = HashSet::new();
    let mut bytes = 0_u64;
    checkout.disable_filters(true);
    checkout.notify_on(CheckoutNotificationType::all());
    checkout.notify(move |_, path, _, target, _| {
        let Some(path) = path else {
            return false;
        };
        if path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        {
            return false;
        }
        if !paths.insert(path.to_path_buf()) {
            return true;
        }
        let size = target.map(|file| file.size()).unwrap_or_default();
        if paths.len() > super::managed_tree::MAX_ENTRIES
            || size > super::managed_tree::MAX_FILE_BYTES
        {
            return false;
        }
        let Some(total) = bytes.checked_add(size) else {
            return false;
        };
        bytes = total;
        bytes <= super::managed_tree::MAX_TOTAL_BYTES
    });
    checkout
}

fn has_runtime_dependencies(root: &Path) -> Result<bool, String> {
    let path = root.join("package.json");
    if !path.is_file() {
        return Ok(false);
    }
    let metadata =
        std::fs::metadata(&path).map_err(|_| "Package d'extension invalide.".to_string())?;
    if metadata.len() > super::types::MAX_MESSAGE_BYTES as u64 {
        return Err("Package d'extension trop volumineux.".to_string());
    }
    let value: Value = serde_json::from_slice(
        &std::fs::read(path).map_err(|_| "Package d'extension invalide.".to_string())?,
    )
    .map_err(|_| "Package d'extension invalide.".to_string())?;
    Ok(["dependencies", "optionalDependencies", "peerDependencies"]
        .into_iter()
        .any(|field| {
            value
                .get(field)
                .and_then(Value::as_object)
                .is_some_and(|dependencies| !dependencies.is_empty())
        }))
}
