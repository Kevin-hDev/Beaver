use super::{
    action_error::GitActionError, branch, branch_commit, branch_merge, repo as git_repo, status,
};
use git2::{build::CheckoutBuilder, BranchType};
use serde::Serialize;
use std::path::Path;

const MAX_UNMERGED_COMMITS: usize = 10_000;

#[derive(Debug, Clone, Serialize)]
pub struct BranchDeletePreview {
    pub branch: String,
    pub is_current: bool,
    pub fallback_branch: Option<String>,
    pub dirty_files: Vec<status::DirtyFile>,
    pub unmerged_commits: usize,
}

pub fn preview(repo_path: &Path, branch_name: &str) -> Result<BranchDeletePreview, GitActionError> {
    branch::validate_branch_name(branch_name).map_err(|_| GitActionError::BranchUnavailable)?;
    let repo = git_repo::open(repo_path).map_err(|_| GitActionError::RepositoryUnavailable)?;
    let target = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(|_| GitActionError::BranchUnavailable)?;
    let current = current_branch_name(&repo);
    let is_current = current.as_deref() == Some(branch_name);
    let fallback_branch = choose_fallback(repo_path, branch_name, current.as_deref())?;
    let dirty_files = if is_current {
        status::list_dirty_files(repo_path).map_err(|_| GitActionError::InternalError)?
    } else {
        Vec::new()
    };
    let unmerged_commits = fallback_branch
        .as_deref()
        .map(|fallback| count_unmerged(&repo, &target, fallback))
        .transpose()?
        .unwrap_or(0);
    Ok(BranchDeletePreview {
        branch: branch_name.to_string(),
        is_current,
        fallback_branch,
        dirty_files,
        unmerged_commits,
    })
}

pub fn discard_and_delete(repo_path: &Path, branch_name: &str) -> Result<(), GitActionError> {
    let deletion = preview(repo_path, branch_name)?;
    if deletion.is_current {
        let fallback = deletion
            .fallback_branch
            .ok_or(GitActionError::NoFallbackBranch)?;
        discard_changes(repo_path)?;
        branch::checkout_branch(repo_path, &fallback)?;
    }
    delete_branch(repo_path, branch_name)
}

pub fn delete_clean(repo_path: &Path, branch_name: &str) -> Result<(), GitActionError> {
    let deletion = preview(repo_path, branch_name)?;
    if !deletion.dirty_files.is_empty() {
        return Err(GitActionError::DirtyWorktree {
            dirty_count: deletion.dirty_files.len(),
        });
    }
    if deletion.unmerged_commits > 0 {
        return Err(GitActionError::UnmergedCommits {
            count: deletion.unmerged_commits,
        });
    }
    if deletion.is_current {
        let fallback = deletion
            .fallback_branch
            .ok_or(GitActionError::NoFallbackBranch)?;
        branch::checkout_branch(repo_path, &fallback)?;
    }
    delete_branch(repo_path, branch_name)
}

pub fn preserve_and_delete(
    repo_path: &Path,
    branch_name: &str,
    description: Option<String>,
) -> Result<(), GitActionError> {
    let deletion = preview(repo_path, branch_name)?;
    let fallback = deletion
        .fallback_branch
        .ok_or(GitActionError::NoFallbackBranch)?;
    if deletion.is_current && !deletion.dirty_files.is_empty() {
        branch_commit::commit_all(repo_path, description)?;
    }
    if deletion.is_current {
        branch::checkout_branch(repo_path, &fallback)?;
    }
    branch_merge::merge_branch(repo_path, branch_name).map_err(|_| GitActionError::MergeFailed)?;
    delete_branch(repo_path, branch_name)
}

pub fn delete_branch(repo_path: &Path, branch_name: &str) -> Result<(), GitActionError> {
    branch::validate_branch_name(branch_name).map_err(|_| GitActionError::BranchUnavailable)?;
    let repo = git_repo::open(repo_path).map_err(|_| GitActionError::RepositoryUnavailable)?;
    let current = repo
        .head()
        .ok()
        .and_then(|head| head.shorthand().ok().map(str::to_string));
    if current.as_deref() == Some(branch_name) {
        return Err(GitActionError::BranchActive);
    }
    let mut branch = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(|_| GitActionError::BranchUnavailable)?;
    branch.delete().map_err(|_| GitActionError::DeleteFailed)
}

pub fn branch_exists(repo_path: &Path, branch_name: &str) -> Result<bool, GitActionError> {
    branch::validate_branch_name(branch_name).map_err(|_| GitActionError::BranchUnavailable)?;
    let repo = git_repo::open(repo_path).map_err(|_| GitActionError::RepositoryUnavailable)?;
    let exists = repo.find_branch(branch_name, BranchType::Local).is_ok();
    Ok(exists)
}

fn current_branch_name(repo: &git2::Repository) -> Option<String> {
    repo.head()
        .ok()
        .and_then(|head| head.shorthand().ok().map(str::to_string))
}

fn choose_fallback(
    repo_path: &Path,
    branch_name: &str,
    current: Option<&str>,
) -> Result<Option<String>, GitActionError> {
    if let Some(current) = current {
        if current != branch_name && branch_exists(repo_path, current)? {
            return Ok(Some(current.to_string()));
        }
    }
    let branches = branch::list_branches(repo_path).map_err(|_| GitActionError::InternalError)?;
    for preferred in ["main", "master", "develop", "dev"] {
        if preferred != branch_name && branches.iter().any(|item| item.name == preferred) {
            return Ok(Some(preferred.to_string()));
        }
    }
    Ok(branches
        .into_iter()
        .find(|item| item.name != branch_name)
        .map(|item| item.name))
}

fn count_unmerged(
    repo: &git2::Repository,
    target: &git2::Branch<'_>,
    fallback: &str,
) -> Result<usize, GitActionError> {
    let target_oid = target
        .get()
        .target()
        .ok_or(GitActionError::BranchUnavailable)?;
    let fallback_oid = repo
        .find_branch(fallback, BranchType::Local)
        .ok()
        .and_then(|branch| branch.get().target())
        .ok_or(GitActionError::NoFallbackBranch)?;
    let mut walk = repo.revwalk().map_err(|_| GitActionError::InternalError)?;
    walk.push(target_oid)
        .map_err(|_| GitActionError::InternalError)?;
    walk.hide(fallback_oid)
        .map_err(|_| GitActionError::InternalError)?;
    let mut count = 0;
    for oid in walk.take(MAX_UNMERGED_COMMITS) {
        oid.map_err(|_| GitActionError::InternalError)?;
        count += 1;
    }
    Ok(count)
}

fn discard_changes(repo_path: &Path) -> Result<(), GitActionError> {
    let repo = git_repo::open(repo_path).map_err(|_| GitActionError::RepositoryUnavailable)?;
    let mut checkout = CheckoutBuilder::new();
    checkout.force().remove_untracked(true);
    repo.checkout_head(Some(&mut checkout))
        .map_err(|_| GitActionError::DeleteFailed)
}
