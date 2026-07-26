use git2::{BranchType, Oid, Repository, RepositoryState};
use serde::Serialize;
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};

pub use super::branch_merge_error::MergeError;
use super::{branch, branch_commit, repo as git_repo, status};

#[derive(Debug, Clone, Serialize)]
pub struct BranchMergePreview {
    pub source_branch: String,
    pub target_branch: String,
    pub commits: usize,
    pub dirty_files: Vec<status::DirtyFile>,
}

pub fn preview(
    repo_path: &Path,
    source_branch: &str,
    expected_target: &str,
) -> Result<BranchMergePreview, MergeError> {
    validate_names(source_branch, expected_target)?;
    let repo = git_repo::open(repo_path).map_err(|_| MergeError::InternalError)?;
    ensure_current_branch(&repo, expected_target)?;
    ensure_repository_ready(&repo)?;
    let source = repo
        .find_branch(source_branch, BranchType::Local)
        .map_err(|_| MergeError::BranchUnavailable)?;
    let source_oid = source.get().target().ok_or(MergeError::BranchUnavailable)?;
    let target_oid = repo
        .head()
        .ok()
        .and_then(|head| head.target())
        .ok_or(MergeError::InternalError)?;
    let (commits, _) = repo
        .graph_ahead_behind(source_oid, target_oid)
        .map_err(|_| MergeError::InternalError)?;
    let dirty_files = status::list_dirty_files(repo_path).map_err(|_| MergeError::InternalError)?;

    Ok(BranchMergePreview {
        source_branch: source_branch.to_string(),
        target_branch: expected_target.to_string(),
        commits,
        dirty_files,
    })
}

pub fn merge_current(
    repo_path: &Path,
    source_branch: &str,
    expected_target: &str,
    commit_changes: bool,
    commit_description: Option<String>,
) -> Result<(), MergeError> {
    let state = preview(repo_path, source_branch, expected_target)?;
    if state.commits == 0 {
        return Err(MergeError::NothingToMerge);
    }
    if !state.dirty_files.is_empty() {
        if !commit_changes {
            return Err(MergeError::DirtyWorktree);
        }
        branch_commit::commit_all(repo_path, commit_description)
            .map_err(|_| MergeError::InternalError)?;
    }
    let original_head = current_head_oid(repo_path, expected_target)?;
    run_merge(repo_path, source_branch, original_head)
}

pub(super) fn merge_branch(repo_path: &Path, source_branch: &str) -> Result<(), String> {
    let repo = git_repo::open(repo_path)?;
    let target = repo
        .head()
        .ok()
        .and_then(|head| head.shorthand().ok().map(str::to_string))
        .ok_or_else(|| "Fusion impossible".to_string())?;
    drop(repo);
    match merge_current(repo_path, source_branch, &target, false, None) {
        Ok(()) | Err(MergeError::NothingToMerge) => Ok(()),
        Err(_) => Err("Fusion impossible".to_string()),
    }
}

fn validate_names(source: &str, target: &str) -> Result<(), MergeError> {
    branch::validate_branch_name(source).map_err(|_| MergeError::BranchUnavailable)?;
    branch::validate_branch_name(target).map_err(|_| MergeError::ContextChanged)?;
    if source == target {
        return Err(MergeError::NothingToMerge);
    }
    Ok(())
}

fn ensure_current_branch(repo: &Repository, expected: &str) -> Result<(), MergeError> {
    let is_expected = repo
        .find_branch(expected, BranchType::Local)
        .map(|branch| branch.is_head())
        .unwrap_or(false);
    if is_expected {
        Ok(())
    } else {
        Err(MergeError::ContextChanged)
    }
}

fn current_head_oid(repo_path: &Path, expected: &str) -> Result<Oid, MergeError> {
    let repo = git_repo::open(repo_path).map_err(|_| MergeError::InternalError)?;
    ensure_current_branch(&repo, expected)?;
    ensure_repository_ready(&repo)?;
    repo.head()
        .ok()
        .and_then(|head| head.target())
        .ok_or(MergeError::InternalError)
}

fn ensure_repository_ready(repo: &Repository) -> Result<(), MergeError> {
    if repo.state() == RepositoryState::Clean {
        Ok(())
    } else {
        Err(MergeError::InternalError)
    }
}

fn run_merge(repo_path: &Path, source: &str, original_head: Oid) -> Result<(), MergeError> {
    let hooks = tempfile::tempdir().map_err(|_| MergeError::InternalError)?;
    let status = git_command(repo_path, hooks.path())
        .args(["merge", "--no-edit", "--no-verify", "--no-gpg-sign", source])
        .status()
        .map_err(|_| MergeError::InternalError)?;
    if status.success() {
        return Ok(());
    }
    abort_and_verify(repo_path, hooks.path(), original_head)?;
    Err(MergeError::MergeConflict)
}

fn git_command(repo_path: &Path, hooks_path: &Path) -> Command {
    let mut hook_config = OsString::from("core.hooksPath=");
    hook_config.push(hooks_path);
    let mut command = Command::new("git");
    command
        .arg("-c")
        .arg(hook_config)
        .arg("-c")
        .arg("commit.gpgSign=false")
        .arg("-C")
        .arg(repo_path)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

fn abort_and_verify(
    repo_path: &Path,
    hooks_path: &Path,
    original_head: Oid,
) -> Result<(), MergeError> {
    let merge_started = git_repo::open(repo_path)
        .map_err(|_| MergeError::InternalError)?
        .path()
        .join("MERGE_HEAD")
        .exists();
    if merge_started {
        let aborted = git_command(repo_path, hooks_path)
            .args(["merge", "--abort"])
            .status()
            .map_err(|_| MergeError::InternalError)?;
        if !aborted.success() {
            return Err(MergeError::InternalError);
        }
    }
    let repo = git_repo::open(repo_path).map_err(|_| MergeError::InternalError)?;
    let restored = repo.head().ok().and_then(|head| head.target()) == Some(original_head);
    let state_restored = repo.state() == RepositoryState::Clean;
    let clean = status::list_dirty_files(repo_path)
        .map_err(|_| MergeError::InternalError)?
        .is_empty();
    if restored && state_restored && clean {
        Ok(())
    } else {
        Err(MergeError::InternalError)
    }
}
