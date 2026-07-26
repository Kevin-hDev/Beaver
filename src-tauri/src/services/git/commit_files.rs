use super::history::{find_reachable_commit, open_current_branch};
use git2::{Delta, DiffFindOptions, Patch};
use serde::Serialize;
use std::path::Path;

const MAX_COMMIT_FILES: usize = 200;
const MAX_PATH_LEN: usize = 4096;

#[derive(Debug, Clone, Serialize)]
pub struct CommitFile {
    pub path: String,
    pub previous_path: Option<String>,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
}

pub fn list_commit_files(
    repo_path: &Path,
    expected_branch: &str,
    commit_id: &str,
) -> Result<Vec<CommitFile>, String> {
    let (repo, head) = open_current_branch(repo_path, expected_branch)?;
    let commit = find_reachable_commit(&repo, head, commit_id)?;
    let tree = commit.tree().map_err(|_| unavailable())?;
    let parent_tree = commit.parent(0).ok().and_then(|parent| parent.tree().ok());
    let mut diff = repo
        .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)
        .map_err(|_| unavailable())?;
    let mut find = DiffFindOptions::new();
    find.renames(true).copies(false);
    diff.find_similar(Some(&mut find))
        .map_err(|_| unavailable())?;

    let mut files = Vec::new();
    for (index, delta) in diff.deltas().take(MAX_COMMIT_FILES).enumerate() {
        let Some((path, previous_path, status)) = describe_delta(&delta) else {
            continue;
        };
        if !valid_path(&path)
            || previous_path
                .as_deref()
                .is_some_and(|value| !valid_path(value))
        {
            continue;
        }
        let (additions, deletions) = patch_stats(&diff, index);
        files.push(CommitFile {
            path,
            previous_path,
            status: status.to_string(),
            additions,
            deletions,
        });
    }
    Ok(files)
}

fn describe_delta(delta: &git2::DiffDelta<'_>) -> Option<(String, Option<String>, &'static str)> {
    let old_path = delta.old_file().path().and_then(Path::to_str);
    let new_path = delta.new_file().path().and_then(Path::to_str);
    match delta.status() {
        Delta::Added => Some((new_path?.to_string(), None, "added")),
        Delta::Deleted => Some((old_path?.to_string(), None, "deleted")),
        Delta::Renamed => Some((
            new_path?.to_string(),
            old_path.map(str::to_string),
            "renamed",
        )),
        Delta::Copied => Some((
            new_path?.to_string(),
            old_path.map(str::to_string),
            "copied",
        )),
        Delta::Modified => Some((new_path?.to_string(), None, "modified")),
        Delta::Typechange => Some((new_path.or(old_path)?.to_string(), None, "changed")),
        _ => None,
    }
}

fn patch_stats(diff: &git2::Diff<'_>, index: usize) -> (u32, u32) {
    let stats = Patch::from_diff(diff, index)
        .ok()
        .flatten()
        .and_then(|patch| patch.line_stats().ok());
    match stats {
        Some((_, additions, deletions)) => (
            u32::try_from(additions).unwrap_or(u32::MAX),
            u32::try_from(deletions).unwrap_or(u32::MAX),
        ),
        None => (0, 0),
    }
}

fn valid_path(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_PATH_LEN && !value.contains('\0')
}

fn unavailable() -> String {
    "Fichiers du commit indisponibles".to_string()
}
