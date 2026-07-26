use serde::Serialize;
use std::path::Path;

use super::{remote_target, repo};

#[derive(Debug, Clone, Serialize)]
pub struct RemoteStatus {
    pub has_remote: bool,
    pub is_github: bool,
    pub has_upstream: bool,
    pub has_remote_branch: bool,
    pub ahead: usize,
    pub behind: usize,
}

pub fn status(repo_path: &Path) -> Result<RemoteStatus, String> {
    let repository = repo::open(repo_path)?;
    let has_remote = repository
        .remotes()
        .map(|items| !items.is_empty())
        .unwrap_or(false);
    let Some(branch) = remote_target::current_branch(&repository).ok() else {
        return Ok(empty_status(has_remote));
    };
    let name = remote_target::branch_name(&branch)?;
    let is_github = remote_target::selected_remote_url(&repository, &name)
        .is_some_and(|url| repo::is_github_url(&url));
    let local_oid = branch
        .get()
        .target()
        .ok_or_else(|| "Référence locale invalide".to_string())?;

    if let Ok(upstream) = branch.upstream() {
        let remote_oid = upstream
            .get()
            .target()
            .ok_or_else(|| "Référence distante invalide".to_string())?;
        return counts(
            &repository,
            local_oid,
            remote_oid,
            has_remote,
            is_github,
            true,
        );
    }

    let Some(remote_oid) = cached_remote_oid(&repository, &name) else {
        return Ok(RemoteStatus {
            has_remote,
            is_github,
            has_upstream: false,
            has_remote_branch: false,
            ahead: 0,
            behind: 0,
        });
    };
    counts(
        &repository,
        local_oid,
        remote_oid,
        has_remote,
        is_github,
        false,
    )
}

fn cached_remote_oid(repository: &git2::Repository, branch: &str) -> Option<git2::Oid> {
    let remote = remote_target::choose_remote(repository, branch)?;
    repository
        .find_reference(&format!("refs/remotes/{remote}/{branch}"))
        .ok()?
        .target()
}

fn counts(
    repository: &git2::Repository,
    local_oid: git2::Oid,
    remote_oid: git2::Oid,
    has_remote: bool,
    is_github: bool,
    has_upstream: bool,
) -> Result<RemoteStatus, String> {
    let (ahead, behind) = repository
        .graph_ahead_behind(local_oid, remote_oid)
        .map_err(|_| "Statut distant indisponible".to_string())?;
    Ok(RemoteStatus {
        has_remote,
        is_github,
        has_upstream,
        has_remote_branch: true,
        ahead,
        behind,
    })
}

fn empty_status(has_remote: bool) -> RemoteStatus {
    RemoteStatus {
        has_remote,
        is_github: false,
        has_upstream: false,
        has_remote_branch: false,
        ahead: 0,
        behind: 0,
    }
}
