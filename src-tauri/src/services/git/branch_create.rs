use super::{github_auth, repo as git_repo};
use git2::{BranchType, ErrorCode};
use serde::Serialize;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum CreateBranchError {
    #[error("invalid branch name")]
    InvalidName,
    #[error("branch name too long")]
    NameTooLong,
    #[error("branch already exists")]
    AlreadyExists,
    #[error("repository has no commit")]
    UnbornHead,
    #[error("github authentication required")]
    GithubAuthRequired,
    #[error("internal git error")]
    InternalError,
}

pub fn create_branch(repo_path: &Path, branch_name: &str) -> Result<(), CreateBranchError> {
    validate_branch_name(branch_name)?;

    let repo = git_repo::open(repo_path).map_err(|_| CreateBranchError::InternalError)?;
    github_auth::ensure_branch_creation_allowed(&repo).map_err(|err| {
        if err == github_auth::GITHUB_AUTH_REQUIRED {
            CreateBranchError::GithubAuthRequired
        } else {
            CreateBranchError::InternalError
        }
    })?;

    if repo
        .find_branch(branch_name, BranchType::Local)
        .map(|_| true)
        .unwrap_or(false)
    {
        return Err(CreateBranchError::AlreadyExists);
    }

    if repo
        .is_empty()
        .map_err(|_| CreateBranchError::InternalError)?
    {
        return Err(CreateBranchError::UnbornHead);
    }

    let head_commit = repo
        .head()
        .map_err(|err| {
            if err.code() == ErrorCode::UnbornBranch {
                CreateBranchError::UnbornHead
            } else {
                CreateBranchError::InternalError
            }
        })?
        .peel_to_commit()
        .map_err(|_| CreateBranchError::InternalError)?;

    repo.branch(branch_name, &head_commit, false)
        .map_err(|_| CreateBranchError::InternalError)?;

    repo.set_head(&format!("refs/heads/{branch_name}"))
        .map_err(|_| CreateBranchError::InternalError)?;

    Ok(())
}

// KEEP IN SYNC with src/lib/branch-name.ts::validateBranchName.
pub fn validate_branch_name(name: &str) -> Result<(), CreateBranchError> {
    if name.chars().count() > 100 {
        return Err(CreateBranchError::NameTooLong);
    }
    if name.is_empty()
        || name.starts_with('-')
        || name.contains("..")
        || name.contains('\0')
        || name.contains('\\')
        || name.contains(':')
        || name.contains('~')
        || name.contains('^')
        || name.contains('?')
        || name.contains('*')
        || name.contains('[')
        || name.contains("@{")
        || name.contains("//")
        || name.starts_with('/')
        || name.ends_with('/')
        || name.ends_with(".lock")
        || name.as_bytes().iter().any(|&b| b <= 0x20 || b == 0x7f)
    {
        return Err(CreateBranchError::InvalidName);
    }
    for segment in name.split('/') {
        if segment.is_empty() || segment.starts_with('.') || segment.ends_with('.') {
            return Err(CreateBranchError::InvalidName);
        }
    }
    Ok(())
}
