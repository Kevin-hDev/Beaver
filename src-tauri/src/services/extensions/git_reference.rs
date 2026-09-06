use super::{
    git_source::looks_like_short_commit, git_transport::fetch_options,
    install_signal::InstallSignal,
};
use git2::Repository;
use std::time::Instant;

pub(super) fn checkout_reference(
    repository: &Repository,
    reference: &str,
    deadline: Instant,
    cancellation: &impl InstallSignal,
) -> Result<(), git2::Error> {
    if let Some(commit) = resolve_commit(repository, reference) {
        repository.checkout_tree(
            commit.as_object(),
            Some(&mut super::git_checkout::bounded(cancellation.clone())),
        )?;
        return repository.set_head_detached(commit.id());
    }
    let mut remote = repository.find_remote("origin")?;
    let mut fetch = fetch_options(deadline, true, cancellation.clone())?;
    let targeted = remote.fetch(&[reference], Some(&mut fetch), None);
    if let Some(commit) = resolve_commit(repository, reference) {
        return checkout_commit(repository, &commit, cancellation);
    }
    if looks_like_short_commit(reference) {
        let mut complete = fetch_options(deadline, false, cancellation.clone())?;
        remote.fetch(
            &[
                "+refs/heads/*:refs/remotes/origin/*",
                "+refs/tags/*:refs/tags/*",
            ],
            Some(&mut complete),
            None,
        )?;
    } else {
        targeted?;
    }
    let commit = resolve_commit(repository, reference)
        .ok_or_else(|| git2::Error::from_str("git reference unavailable"))?;
    checkout_commit(repository, &commit, cancellation)
}

fn checkout_commit(
    repository: &Repository,
    commit: &git2::Commit<'_>,
    cancellation: &impl InstallSignal,
) -> Result<(), git2::Error> {
    repository.checkout_tree(
        commit.as_object(),
        Some(&mut super::git_checkout::bounded(cancellation.clone())),
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
