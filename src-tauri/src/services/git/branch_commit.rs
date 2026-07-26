use git2::{BranchType, IndexAddOption, Repository, Signature};
use std::path::Path;
use std::process::Command;

use super::{action_error::GitActionError, branch, branch_index_backup::IndexBackup, repo};
use crate::services::brand::DISPLAY_NAME;

const MAX_COMMIT_DESCRIPTION_CHARS: usize = 2_000;

pub fn commit_all(repo_path: &Path, description: Option<String>) -> Result<(), GitActionError> {
    let git_repo = repo::open(repo_path).map_err(|_| GitActionError::RepositoryUnavailable)?;
    let workdir = repo::workdir(&git_repo).map_err(|_| GitActionError::RepositoryUnavailable)?;
    let dirty = branch::count_dirty_files(&git_repo).map_err(|_| GitActionError::InternalError)?;
    if dirty == 0 {
        return Ok(());
    }

    let description = sanitize_description(description)?;
    let signature = commit_signature(&git_repo, &workdir)?;
    let index_backup = IndexBackup::capture(&git_repo)?;
    let message = description.unwrap_or_else(|| format!("Save changes from {DISPLAY_NAME}"));
    if let Err(error) = create_commit(&git_repo, &message, &signature) {
        index_backup.restore();
        return Err(error);
    }
    Ok(())
}

pub fn commit_all_and_checkout(
    repo_path: &Path,
    target_branch: &str,
    description: Option<String>,
) -> Result<(), GitActionError> {
    branch::validate_branch_name(target_branch).map_err(|_| GitActionError::BranchUnavailable)?;
    let git_repo = repo::open(repo_path).map_err(|_| GitActionError::RepositoryUnavailable)?;
    let workdir = repo::workdir(&git_repo).map_err(|_| GitActionError::RepositoryUnavailable)?;
    ensure_local_branch_exists(&git_repo, target_branch)?;

    let dirty = branch::count_dirty_files(&git_repo).map_err(|_| GitActionError::InternalError)?;
    if dirty == 0 {
        return branch::checkout_branch(repo_path, target_branch);
    }

    let description = sanitize_description(description)?;
    let signature = commit_signature(&git_repo, &workdir)?;
    let index_backup = IndexBackup::capture(&git_repo)?;
    if let Err(e) = create_wip_commit(&git_repo, target_branch, description.as_deref(), &signature)
    {
        index_backup.restore();
        return Err(e);
    }

    branch::checkout_branch(repo_path, target_branch)
}

fn ensure_local_branch_exists(repo: &Repository, branch_name: &str) -> Result<(), GitActionError> {
    repo.find_branch(branch_name, BranchType::Local)
        .map(|_| ())
        .map_err(|_| GitActionError::BranchUnavailable)
}

fn create_wip_commit(
    repo: &Repository,
    target_branch: &str,
    description: Option<&str>,
    signature: &Signature<'_>,
) -> Result<(), GitActionError> {
    let message = build_commit_message(target_branch, description);
    create_commit(repo, &message, signature)
}

fn create_commit(
    repo: &Repository,
    message: &str,
    signature: &Signature<'_>,
) -> Result<(), GitActionError> {
    let mut index = repo.index().map_err(|_| GitActionError::CommitFailed)?;
    index
        .add_all(["*"], IndexAddOption::DEFAULT, None)
        .map_err(|_| GitActionError::CommitFailed)?;
    index.write().map_err(|_| GitActionError::CommitFailed)?;

    let tree_oid = index
        .write_tree()
        .map_err(|_| GitActionError::CommitFailed)?;
    let tree = repo
        .find_tree(tree_oid)
        .map_err(|_| GitActionError::CommitFailed)?;
    let parent = repo
        .head()
        .and_then(|h| h.peel_to_commit())
        .map_err(|_| GitActionError::CommitFailed)?;
    repo.commit(
        Some("HEAD"),
        signature,
        signature,
        message,
        &tree,
        &[&parent],
    )
    .map_err(|_| GitActionError::CommitFailed)?;
    Ok(())
}

pub(super) fn build_commit_message(target_branch: &str, description: Option<&str>) -> String {
    let subject = format!("WIP: save changes before switching to {target_branch}");
    match description {
        Some(body) if !body.is_empty() => format!("{subject}\n\n{body}"),
        _ => subject,
    }
}

pub(super) fn sanitize_description(
    description: Option<String>,
) -> Result<Option<String>, GitActionError> {
    let Some(description) = description else {
        return Ok(None);
    };
    let normalized = description.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > MAX_COMMIT_DESCRIPTION_CHARS
        || trimmed
            .chars()
            .any(|c| c == '\0' || (c.is_control() && c != '\n' && c != '\t'))
    {
        return Err(GitActionError::InvalidCommitDescription);
    }
    Ok(Some(trimmed.to_string()))
}

fn commit_signature(
    repo: &Repository,
    workdir: &Path,
) -> Result<Signature<'static>, GitActionError> {
    if let Ok(sig) = repo.signature() {
        return Ok(sig);
    }
    signature_from_git_var(workdir).ok_or(GitActionError::IdentityMissing)
}

fn signature_from_git_var(workdir: &Path) -> Option<Signature<'static>> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(workdir)
        .args(["var", "GIT_AUTHOR_IDENT"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let ident = String::from_utf8(output.stdout).ok()?;
    let (name, email) = parse_git_ident(&ident)?;
    Signature::now(&name, &email).ok()
}

pub(super) fn parse_git_ident(ident: &str) -> Option<(String, String)> {
    let start = ident.rfind('<')?;
    let end = ident[start..].find('>')? + start;
    let name = ident[..start].trim();
    let email = ident[start + 1..end].trim();
    if name.is_empty() || email.is_empty() {
        return None;
    }
    Some((name.to_string(), email.to_string()))
}
