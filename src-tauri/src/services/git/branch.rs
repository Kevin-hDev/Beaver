use super::{action_error::GitActionError, repo as git_repo};
use git2::{BranchType, StatusOptions};
use serde::Serialize;
use std::path::Path;

const MAX_BRANCHES: usize = 500;

pub use super::branch_create::{create_branch, validate_branch_name, CreateBranchError};

#[derive(Debug, Clone, Serialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub dirty_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitContext {
    pub branch: String,
    pub is_detached: bool,
    pub dirty_count: usize,
    pub is_git_repo: bool,
}

pub fn list_branches(repo_path: &Path) -> Result<Vec<BranchInfo>, String> {
    let repo = git_repo::open(repo_path)?;

    let current_name = repo
        .head()
        .ok()
        .and_then(|head| head.shorthand().ok().map(String::from));

    let dirty_count = count_dirty_files(&repo).unwrap_or(0);
    let mut branches = Vec::new();

    for branch_result in repo
        .branches(Some(BranchType::Local))
        .map_err(|e| format!("Lecture des branches : {e}"))?
    {
        if branches.len() >= MAX_BRANCHES {
            break;
        }
        let (branch, _) = branch_result.map_err(|e| format!("Lecture branche : {e}"))?;
        let name = branch
            .name()
            .map_err(|e| format!("Nom de branche : {e}"))?
            .unwrap_or("?")
            .to_string();
        let is_current = current_name.as_deref() == Some(name.as_str());

        branches.push(BranchInfo {
            name,
            is_current,
            is_remote: false,
            dirty_count: if is_current { dirty_count } else { 0 },
        });
    }

    branches.sort_by(|a, b| b.is_current.cmp(&a.is_current).then(a.name.cmp(&b.name)));

    Ok(branches)
}

pub(super) fn count_dirty_files(repo: &git2::Repository) -> Result<usize, git2::Error> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(false);
    let statuses = repo.statuses(Some(&mut opts))?;
    Ok(statuses.len())
}

pub fn get_context(repo_path: &Path) -> GitContext {
    let repo = match git_repo::open(repo_path) {
        Ok(r) => r,
        Err(_) => {
            return GitContext {
                branch: String::new(),
                is_detached: false,
                dirty_count: 0,
                is_git_repo: false,
            }
        }
    };

    let is_detached = repo.head_detached().unwrap_or(false);
    let branch = repo
        .head()
        .ok()
        .and_then(|head| head.shorthand().ok().map(String::from))
        .unwrap_or_else(|| "HEAD".to_string());
    let dirty_count = count_dirty_files(&repo).unwrap_or(0);

    GitContext {
        branch,
        is_detached,
        dirty_count,
        is_git_repo: true,
    }
}

pub fn checkout_branch(repo_path: &Path, branch_name: &str) -> Result<(), GitActionError> {
    validate_branch_name(branch_name).map_err(|_| GitActionError::BranchUnavailable)?;

    let repo = git_repo::open(repo_path).map_err(|_| GitActionError::RepositoryUnavailable)?;
    let (object, reference) = repo
        .revparse_ext(&format!("refs/heads/{branch_name}"))
        .map_err(|_| GitActionError::BranchUnavailable)?;

    let dirty = count_dirty_files(&repo).map_err(|_| GitActionError::InternalError)?;
    if dirty > 0 {
        return Err(GitActionError::DirtyWorktree { dirty_count: dirty });
    }

    repo.checkout_tree(&object, None)
        .map_err(|_| GitActionError::CheckoutFailed)?;

    if let Some(refname) = reference
        .as_ref()
        .and_then(|reference| reference.name().ok())
    {
        repo.set_head(refname)
            .map_err(|_| GitActionError::CheckoutFailed)?;
    }

    Ok(())
}
