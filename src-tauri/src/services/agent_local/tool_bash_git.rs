use git2::{Repository, Status, StatusOptions};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use super::tool_file_changes::{capture, FileState, MAX_FILE_CHANGE_DIFF_BYTES, MAX_FILE_CHANGES};
use super::types_tools::ToolFileChangeStatus;

const MAX_PATH_BYTES: usize = 4_096;

pub struct GitBaseline {
    repository: PathBuf,
    workdir: PathBuf,
    head_tree: Option<git2::Oid>,
    initial: BTreeMap<PathBuf, Option<FileState>>,
    incomplete: bool,
}

impl GitBaseline {
    pub fn capture(root: &Path) -> (Option<Self>, bool) {
        let Ok(root) = dunce::canonicalize(root) else {
            return (None, true);
        };
        let repository = match Repository::discover(&root) {
            Ok(repository) => repository,
            Err(error) if error.code() == git2::ErrorCode::NotFound => return (None, false),
            Err(_) => return (None, true),
        };
        let Some(workdir) = repository
            .workdir()
            .and_then(|workdir| dunce::canonicalize(workdir).ok())
        else {
            return (None, true);
        };
        let head_tree = repository
            .head()
            .ok()
            .and_then(|head| head.peel_to_tree().ok())
            .map(|tree| tree.id());
        let (paths, incomplete) = status_paths(&repository, &root);
        let mut remaining = MAX_FILE_CHANGE_DIFF_BYTES;
        let initial = paths
            .into_iter()
            .map(|(path, _)| {
                let state = capture(&path, &mut remaining);
                (path, state)
            })
            .collect();
        (
            Some(Self {
                repository: repository.path().to_path_buf(),
                workdir,
                head_tree,
                initial,
                incomplete,
            }),
            false,
        )
    }

    pub fn is_incomplete(&self) -> bool {
        self.incomplete
    }

    pub fn open_repository(&self) -> Option<Repository> {
        Repository::open(&self.repository).ok()
    }

    pub fn before_state(
        &self,
        repository: Option<&Repository>,
        path: &Path,
    ) -> Option<Option<FileState>> {
        let normalized;
        let path = match dunce::canonicalize(path) {
            Ok(canonical) => {
                normalized = canonical;
                normalized.as_path()
            }
            Err(_) => path,
        };
        if let Some(state) = self.initial.get(path) {
            return Some(state.clone());
        }
        let relative = path.strip_prefix(&self.workdir).ok()?;
        let tree_id = match self.head_tree {
            Some(tree_id) => tree_id,
            None => return Some(None),
        };
        let repository = repository?;
        let tree = repository.find_tree(tree_id).ok()?;
        let entry = match tree.get_path(relative) {
            Ok(entry) => entry,
            Err(error) if error.code() == git2::ErrorCode::NotFound => return Some(None),
            Err(_) => return None,
        };
        let blob = repository.find_blob(entry.id()).ok()?;
        Some(Some(super::tool_file_changes::baseline_state(
            path,
            blob.content(),
        )))
    }

    pub fn current_paths(
        &self,
        repository: &Repository,
        root: &Path,
    ) -> (Vec<(PathBuf, ToolFileChangeStatus)>, bool) {
        status_paths(repository, root)
    }
}

fn status_paths(
    repository: &Repository,
    root: &Path,
) -> (Vec<(PathBuf, ToolFileChangeStatus)>, bool) {
    let Some(workdir) = repository.workdir() else {
        return (Vec::new(), true);
    };
    let mut options = StatusOptions::new();
    options
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);
    let Ok(statuses) = repository.statuses(Some(&mut options)) else {
        return (Vec::new(), true);
    };
    let mut paths = Vec::new();
    let mut incomplete = false;
    for entry in statuses.iter() {
        let Some(relative) = entry.path().ok().and_then(valid_relative_path) else {
            incomplete = true;
            continue;
        };
        let path = workdir.join(relative);
        if !path.starts_with(root)
            || !super::tool_bash_change_hub::is_trackable(root, &path)
        {
            continue;
        }
        if paths.len() >= MAX_FILE_CHANGES {
            incomplete = true;
            break;
        }
        paths.push((path, change_status(entry.status())));
    }
    (paths, incomplete)
}

fn valid_relative_path(value: &str) -> Option<PathBuf> {
    if value.is_empty() || value.len() > MAX_PATH_BYTES || value.contains('\0') {
        return None;
    }
    let path = Path::new(value);
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return None;
    }
    Some(path.to_path_buf())
}

fn change_status(status: Status) -> ToolFileChangeStatus {
    if status.is_wt_new() || status.is_index_new() {
        ToolFileChangeStatus::Added
    } else if status.is_wt_deleted() || status.is_index_deleted() {
        ToolFileChangeStatus::Deleted
    } else {
        ToolFileChangeStatus::Modified
    }
}

#[cfg(test)]
#[path = "tool_bash_git_tests.rs"]
mod tests;
