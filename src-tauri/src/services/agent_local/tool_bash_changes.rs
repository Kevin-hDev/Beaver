use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::tool_bash_change_hub::{workspace_hub, WorkspaceEventHub};
use super::tool_bash_directory_baseline::DirectoryBaseline;
use super::tool_bash_git::GitBaseline;
use super::tool_file_changes::{build_change, capture, MAX_FILE_CHANGE_DIFF_BYTES};
use super::types_tools::{ToolFileChange, ToolFileChangeStatus};

const MAX_TRACKED_PATHS: usize = 500;
const WATCHER_START_TIMEOUT_MS: u64 = 1_000;
const BASELINE_TIMEOUT_MS: u64 = 5_000;

pub struct ChangeTracker {
    root: PathBuf,
    hub: Option<std::sync::Arc<WorkspaceEventHub>>,
    cursor: u64,
    changes: BTreeMap<PathBuf, ToolFileChangeStatus>,
    baseline: Option<GitBaseline>,
    directory_baseline: Option<DirectoryBaseline>,
    baseline_incomplete: bool,
    overflowed: bool,
    pending_update: bool,
}

impl ChangeTracker {
    pub async fn start(root: &Path) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|_| "Suivi des fichiers indisponible.".to_string())?;
        let watcher_root = root.clone();
        let mut watcher_setup =
            tokio::task::spawn_blocking(move || workspace_hub(watcher_root));
        let baseline_root = root.clone();
        let mut baseline_setup =
            tokio::task::spawn_blocking(move || capture_baseline(&baseline_root));
        let hub = tokio::time::timeout(
            std::time::Duration::from_millis(WATCHER_START_TIMEOUT_MS),
            &mut watcher_setup,
        )
        .await
        .ok()
        .and_then(Result::ok)
        .and_then(Result::ok);
        // Blocking tasks cannot be cancelled; dropping only detaches their handles.
        drop(watcher_setup);
        let (baseline, directory_baseline, baseline_incomplete) = match tokio::time::timeout(
            std::time::Duration::from_millis(BASELINE_TIMEOUT_MS),
            &mut baseline_setup,
        )
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => (None, None, true),
            Err(_) => (None, None, true),
        };
        drop(baseline_setup);
        let cursor = hub.as_ref().map_or(0, |hub| hub.sequence());
        let overflowed = baseline_incomplete;
        Ok(Self {
            root,
            hub,
            cursor,
            changes: BTreeMap::new(),
            baseline,
            directory_baseline,
            baseline_incomplete,
            overflowed,
            pending_update: overflowed,
        })
    }

    #[cfg(test)]
    pub fn changes(&mut self) -> Vec<ToolFileChange> {
        self.drain_ready();
        self.snapshot(false, None)
    }

    pub fn updated_changes(&mut self) -> Option<(Vec<ToolFileChange>, bool)> {
        if !self.drain_ready() && !std::mem::take(&mut self.pending_update) {
            return None;
        }
        Some((self.snapshot(false, None), self.overflowed))
    }

    pub fn requires_event_settle(&self) -> bool {
        self.hub.is_some() && self.baseline.is_none() && self.directory_baseline.is_none()
    }

    pub fn finish_changes(&mut self) -> (Vec<ToolFileChange>, bool) {
        self.drain_ready();
        self.overflowed = self.baseline_incomplete;
        let repository = self
            .baseline
            .as_ref()
            .and_then(GitBaseline::open_repository);
        if let Some(baseline) = &self.baseline {
            if let Some(repository) = repository.as_ref() {
                let (paths, incomplete) = baseline.current_paths(repository, &self.root);
                self.overflowed |= incomplete;
                for (path, status) in paths {
                    self.record(path, status);
                }
            } else {
                self.overflowed = true;
            }
        } else if let Some(baseline) = &self.directory_baseline {
            let (paths, incomplete) = baseline.current_paths();
            self.overflowed |= incomplete;
            for (path, status) in paths {
                self.record(path, status);
            }
        }
        (self.snapshot(true, repository.as_ref()), self.overflowed)
    }

    pub(super) fn snapshot(
        &self,
        include_diffs: bool,
        repository: Option<&git2::Repository>,
    ) -> Vec<ToolFileChange> {
        let mut remaining = MAX_FILE_CHANGE_DIFF_BYTES;
        self.changes
            .iter()
            .filter_map(|(path, status)| {
                self.change_for(path, *status, include_diffs, repository, &mut remaining)
            })
            .collect()
    }

    fn change_for(
        &self,
        path: &Path,
        status: ToolFileChangeStatus,
        include_diff: bool,
        repository: Option<&git2::Repository>,
        remaining: &mut usize,
    ) -> Option<ToolFileChange> {
        if !include_diff {
            return Some(metadata_change(path, status));
        }
        if let Some(before) = self
            .baseline
            .as_ref()
            .and_then(|baseline| baseline.before_state(repository, path))
            .or_else(|| {
                self.directory_baseline
                    .as_ref()
                    .and_then(|baseline| baseline.before_state(path))
            })
        {
            let after = capture(path, remaining);
            if before.is_some() || after.is_some() {
                return build_change(path, before.as_ref(), after.as_ref());
            }
            if !path.is_dir() {
                return None;
            }
        }
        Some(metadata_change(path, status))
    }

    pub(super) fn drain_ready(&mut self) -> bool {
        let Some(hub) = &self.hub else {
            return false;
        };
        let (events, gap) = hub.events_after(self.cursor);
        let updated = (gap && !self.overflowed) || !events.is_empty();
        self.overflowed |= gap;
        for event in events {
            self.cursor = self.cursor.max(event.sequence);
            self.record(event.path, event.status);
        }
        updated
    }

    fn record(&mut self, path: PathBuf, next: ToolFileChangeStatus) {
        if !path.starts_with(&self.root) {
            return;
        }
        if !self.changes.contains_key(&path) && self.changes.len() >= MAX_TRACKED_PATHS {
            if let Some(path) = self.changes.keys().next().cloned() {
                self.changes.remove(&path);
            }
            self.overflowed = true;
        }
        match (self.changes.get(&path).copied(), next) {
            (Some(ToolFileChangeStatus::Added), ToolFileChangeStatus::Deleted) => {
                self.changes.remove(&path);
            }
            (Some(ToolFileChangeStatus::Added), _) => {}
            (Some(ToolFileChangeStatus::Deleted), ToolFileChangeStatus::Added) => {
                self.changes.insert(path, ToolFileChangeStatus::Modified);
            }
            _ => {
                self.changes.insert(path, next);
            }
        }
    }
}

fn metadata_change(path: &Path, status: ToolFileChangeStatus) -> ToolFileChange {
    ToolFileChange {
        path: path.to_string_lossy().to_string(),
        status,
        additions: 0,
        deletions: 0,
        diff: None,
    }
}

fn capture_baseline(
    root: &Path,
) -> (Option<GitBaseline>, Option<DirectoryBaseline>, bool) {
    let (git, git_incomplete) = GitBaseline::capture(root);
    if let Some(git) = git {
        let incomplete = git_incomplete || git.is_incomplete();
        return (Some(git), None, incomplete);
    }
    let directory = DirectoryBaseline::capture(root);
    let incomplete = directory.is_incomplete();
    (None, Some(directory), incomplete)
}

#[cfg(test)]
#[path = "tool_bash_changes_tests.rs"]
mod tests;
