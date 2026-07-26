use super::{branch, repo as git_repo, status};
use git2::{Commit, Oid, Repository, Sort};
use serde::Serialize;
use std::path::Path;

const DEFAULT_PAGE_SIZE: usize = 24;
const MAX_PAGE_SIZE: usize = 50;
const MAX_HISTORY_SCAN: usize = 10_000;
const MAX_MESSAGE_CHARS: usize = 160;
const MAX_MESSAGE_BYTES: usize = 640;

#[derive(Debug, Clone, Serialize)]
pub struct CommitSummary {
    pub id: String,
    pub short_id: String,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommitPage {
    pub commits: Vec<CommitSummary>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UncommittedSnapshot {
    pub head_commit: String,
    pub files: Vec<status::DirtyFile>,
    pub total_files: usize,
    pub truncated: bool,
}

pub fn list_commits(
    repo_path: &Path,
    expected_branch: &str,
    cursor: Option<&str>,
    limit: Option<usize>,
) -> Result<CommitPage, String> {
    let (repo, head) = open_current_branch(repo_path, expected_branch)?;
    let cursor_id = cursor
        .map(|value| find_reachable_commit(&repo, head, value).map(|commit| commit.id()))
        .transpose()?;
    let page_size = limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
    let mut walk = repo.revwalk().map_err(|_| unavailable())?;
    walk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)
        .map_err(|_| unavailable())?;
    walk.push(head).map_err(|_| unavailable())?;

    let mut found_cursor = cursor_id.is_none();
    let mut commits = Vec::with_capacity(page_size + 1);
    for (scanned, item) in walk.enumerate() {
        if scanned >= MAX_HISTORY_SCAN {
            return Err(unavailable());
        }
        let oid = item.map_err(|_| unavailable())?;
        if !found_cursor {
            if Some(oid) == cursor_id {
                found_cursor = true;
            }
            continue;
        }
        commits.push(to_summary(
            repo.find_commit(oid).map_err(|_| unavailable())?,
        ));
        if commits.len() > page_size {
            break;
        }
    }
    if !found_cursor {
        return Err(unavailable());
    }
    let has_more = commits.len() > page_size;
    commits.truncate(page_size);
    let next_cursor = has_more
        .then(|| commits.last().map(|commit| commit.id.clone()))
        .flatten();
    Ok(CommitPage {
        commits,
        next_cursor,
    })
}

pub fn list_uncommitted(
    repo_path: &Path,
    expected_branch: &str,
) -> Result<UncommittedSnapshot, String> {
    let (repo, head) = open_current_branch(repo_path, expected_branch)?;
    let dirty = status::list_dirty_files_snapshot(&repo)?;
    let current_head = repo.head().map_err(|_| unavailable())?;
    if !current_head.is_branch()
        || current_head.shorthand().ok() != Some(expected_branch)
        || current_head.target() != Some(head)
    {
        return Err(unavailable());
    }
    Ok(UncommittedSnapshot {
        head_commit: head.to_string(),
        files: dirty.files,
        total_files: dirty.total_files,
        truncated: dirty.truncated,
    })
}

pub(super) fn open_current_branch(
    repo_path: &Path,
    expected_branch: &str,
) -> Result<(Repository, Oid), String> {
    branch::validate_branch_name(expected_branch).map_err(|_| unavailable())?;
    let repo = git_repo::open(repo_path).map_err(|_| unavailable())?;
    let head = repo.head().map_err(|_| unavailable())?;
    if !head.is_branch() || head.shorthand().ok() != Some(expected_branch) {
        return Err(unavailable());
    }
    let oid = head.target().ok_or_else(unavailable)?;
    drop(head);
    Ok((repo, oid))
}

pub(super) fn find_reachable_commit<'repo>(
    repo: &'repo Repository,
    head: Oid,
    commit_id: &str,
) -> Result<Commit<'repo>, String> {
    validate_oid(commit_id)?;
    let oid = Oid::from_str(commit_id).map_err(|_| unavailable())?;
    let commit = repo.find_commit(oid).map_err(|_| unavailable())?;
    let reachable = oid == head || repo.graph_descendant_of(head, oid).unwrap_or(false);
    if !reachable {
        return Err(unavailable());
    }
    Ok(commit)
}

fn validate_oid(value: &str) -> Result<(), String> {
    if value.len() != 40 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(unavailable());
    }
    Ok(())
}

fn to_summary(commit: Commit<'_>) -> CommitSummary {
    let id = commit.id().to_string();
    CommitSummary {
        short_id: id.chars().take(8).collect(),
        id,
        message: bounded_message(&commit),
        timestamp: commit.time().seconds(),
    }
}

fn bounded_message(commit: &Commit<'_>) -> String {
    let first_line = commit
        .message_bytes()
        .split(|byte| *byte == b'\n' || *byte == b'\r')
        .next()
        .unwrap_or_default();
    let bounded: Vec<u8> = first_line.iter().copied().take(MAX_MESSAGE_BYTES).collect();
    String::from_utf8_lossy(&bounded)
        .trim()
        .chars()
        .take(MAX_MESSAGE_CHARS)
        .collect()
}

fn unavailable() -> String {
    "Historique Git indisponible".to_string()
}
