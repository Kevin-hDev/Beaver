use super::repo as git_repo;
use git2::{DiffFindOptions, DiffOptions, Patch, StatusEntry, StatusOptions};
use serde::Serialize;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

const MAX_DIRTY_FILES: usize = 200;
const MAX_PATH_LEN: usize = 4096;

#[derive(Debug, Clone, Serialize)]
pub struct DirtyFile {
    pub path: String,
    pub previous_path: Option<String>,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
}

pub(super) struct DirtyFilesSnapshot {
    pub(super) files: Vec<DirtyFile>,
    pub(super) total_files: usize,
    pub(super) truncated: bool,
}

pub fn list_dirty_files(repo_path: &Path) -> Result<Vec<DirtyFile>, String> {
    let repo = git_repo::open(repo_path)?;
    Ok(list_dirty_files_snapshot(&repo)?.files)
}

pub(super) fn list_dirty_files_snapshot(
    repo: &git2::Repository,
) -> Result<DirtyFilesSnapshot, String> {
    let workdir = git_repo::workdir(repo)?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);

    let statuses = repo
        .statuses(Some(&mut opts))
        .map_err(|e| format!("Lecture du statut : {e}"))?;
    let total_files = statuses.len();

    let diff_stats = get_diff_stats(repo);

    let mut files = Vec::new();
    for entry in statuses.iter() {
        if files.len() >= MAX_DIRTY_FILES {
            break;
        }
        let Some((path, previous_path, label)) = describe_status(&entry) else {
            continue;
        };
        if !valid_path(&path)
            || previous_path
                .as_deref()
                .is_some_and(|value| !valid_path(value))
        {
            continue;
        }

        let (additions, deletions) = diff_stats.get(&path).copied().unwrap_or_else(|| {
            if label == "new" {
                (count_file_lines(&workdir.join(&path)), 0)
            } else {
                (0, 0)
            }
        });

        files.push(DirtyFile {
            path,
            previous_path,
            status: label.to_string(),
            additions,
            deletions,
        });
    }
    let truncated = total_files > files.len();
    Ok(DirtyFilesSnapshot {
        files,
        total_files,
        truncated,
    })
}

pub(super) fn watch_signature(repo: &git2::Repository) -> Result<(usize, u64), String> {
    let workdir = git_repo::workdir(repo)?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);
    let statuses = repo
        .statuses(Some(&mut opts))
        .map_err(|_| "Statut Git indisponible")?;
    let mut hasher = DefaultHasher::new();
    statuses.len().hash(&mut hasher);
    for entry in statuses.iter().take(MAX_DIRTY_FILES) {
        entry.status().bits().hash(&mut hasher);
        let Ok(path) = entry.path() else { continue };
        path.hash(&mut hasher);
        if let Ok(metadata) = workdir.join(path).symlink_metadata() {
            metadata.len().hash(&mut hasher);
            metadata.modified().ok().hash(&mut hasher);
        }
    }
    Ok((statuses.len(), hasher.finish()))
}

fn describe_status(entry: &StatusEntry<'_>) -> Option<(String, Option<String>, &'static str)> {
    let status = entry.status();
    if status.is_index_renamed() || status.is_wt_renamed() {
        let delta = if status.is_wt_renamed() {
            entry.index_to_workdir()
        } else {
            entry.head_to_index()
        }?;
        let old_path = delta.old_file().path()?.to_str()?.to_string();
        let new_path = delta.new_file().path()?.to_str()?.to_string();
        return Some((new_path, Some(old_path), "renamed"));
    }
    let path = entry.path().ok()?.to_string();
    let label = if status.is_index_new() || status.is_wt_new() {
        "new"
    } else if status.is_index_modified() || status.is_wt_modified() {
        "modified"
    } else if status.is_index_deleted() || status.is_wt_deleted() {
        "deleted"
    } else {
        "changed"
    };
    Some((path, None, label))
}

fn get_diff_stats(repo: &git2::Repository) -> HashMap<String, (u32, u32)> {
    let mut map = HashMap::new();
    let Ok(tree) = repo.head().and_then(|head| head.peel_to_tree()) else {
        return map;
    };
    let mut options = DiffOptions::new();
    options
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .show_untracked_content(true);
    let Ok(mut diff) = repo.diff_tree_to_workdir_with_index(Some(&tree), Some(&mut options)) else {
        return map;
    };
    let mut find = DiffFindOptions::new();
    find.renames(true).copies(false).for_untracked(true);
    if diff.find_similar(Some(&mut find)).is_err() {
        return map;
    }
    for (index, delta) in diff.deltas().take(MAX_DIRTY_FILES).enumerate() {
        let Some(path) = delta.new_file().path().or_else(|| delta.old_file().path()) else {
            continue;
        };
        let Some(path) = path.to_str() else { continue };
        if path.is_empty() || path.len() > MAX_PATH_LEN || path.contains('\0') {
            continue;
        }
        let stats = Patch::from_diff(&diff, index)
            .ok()
            .flatten()
            .and_then(|patch| patch.line_stats().ok());
        if let Some((_, additions, deletions)) = stats {
            map.insert(
                path.to_string(),
                (bounded_u32(additions), bounded_u32(deletions)),
            );
        }
    }
    map
}

fn valid_path(path: &str) -> bool {
    !path.is_empty() && path.len() <= MAX_PATH_LEN && !path.contains('\0')
}

fn bounded_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

const MAX_FILE_READ: u64 = 1_048_576;

fn count_file_lines(path: &Path) -> u32 {
    let meta = match path.symlink_metadata() {
        Ok(m) => m,
        Err(_) => return 0,
    };
    if !meta.is_file() || meta.len() > MAX_FILE_READ {
        return 0;
    }
    std::fs::read_to_string(path)
        .map(|c| c.lines().count() as u32)
        .unwrap_or(0)
}
