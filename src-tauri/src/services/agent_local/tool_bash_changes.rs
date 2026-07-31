use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::tool_bash_change_hub::{workspace_hub, WorkspaceEventHub};
use super::tool_bash_git::GitBaseline;
use super::tool_file_changes::{build_change, capture, MAX_FILE_CHANGE_DIFF_BYTES};
use super::types_tools::{ToolFileChange, ToolFileChangeStatus};

const MAX_TRACKED_PATHS: usize = 500;
const WATCHER_START_TIMEOUT_MS: u64 = 1_000;

pub struct ChangeTracker {
    hub: std::sync::Arc<WorkspaceEventHub>,
    cursor: u64,
    changes: BTreeMap<PathBuf, ToolFileChangeStatus>,
    baseline: Option<GitBaseline>,
    overflowed: bool,
    pending_update: bool,
}

impl ChangeTracker {
    pub async fn start(root: &Path) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|_| "Suivi des fichiers indisponible.".to_string())?;
        let setup = tokio::task::spawn_blocking(move || {
            let hub = workspace_hub(root.clone())?;
            let (baseline, git_incomplete) = GitBaseline::capture(&root);
            Ok::<_, String>((hub, baseline, git_incomplete))
        });
        let (hub, baseline, git_incomplete) = tokio::time::timeout(
            std::time::Duration::from_millis(WATCHER_START_TIMEOUT_MS),
            setup,
        )
        .await
        .map_err(|_| "Suivi des fichiers indisponible.".to_string())?
        .map_err(|_| "Suivi des fichiers indisponible.".to_string())??;
        let cursor = hub.sequence();
        let overflowed = hub.is_incomplete()
            || git_incomplete
            || baseline.as_ref().is_some_and(GitBaseline::is_incomplete);
        Ok(Self {
            hub,
            cursor,
            changes: BTreeMap::new(),
            baseline,
            overflowed,
            pending_update: overflowed,
        })
    }

    #[cfg(test)]
    pub fn changes(&mut self) -> Vec<ToolFileChange> {
        self.drain();
        self.snapshot(false)
    }

    pub fn updated_changes(&mut self) -> Option<(Vec<ToolFileChange>, bool)> {
        if !self.drain() && !std::mem::take(&mut self.pending_update) {
            return None;
        }
        Some((self.snapshot(false), self.overflowed))
    }

    pub fn requires_event_settle(&self) -> bool {
        self.baseline.is_none()
    }

    pub fn finish_changes(&mut self) -> (Vec<ToolFileChange>, bool) {
        self.drain();
        if let Some(baseline) = &self.baseline {
            let (paths, incomplete) = baseline.current_paths(self.hub.root());
            self.overflowed |= incomplete;
            for (path, status) in paths {
                self.record(path, status);
            }
        }
        (self.snapshot(true), self.overflowed)
    }

    fn snapshot(&self, include_diffs: bool) -> Vec<ToolFileChange> {
        let mut remaining = MAX_FILE_CHANGE_DIFF_BYTES;
        self.changes
            .iter()
            .filter_map(|(path, status)| {
                self.change_for(path, *status, include_diffs, &mut remaining)
            })
            .collect()
    }

    fn change_for(
        &self,
        path: &Path,
        status: ToolFileChangeStatus,
        include_diff: bool,
        remaining: &mut usize,
    ) -> Option<ToolFileChange> {
        if !include_diff {
            return Some(metadata_change(path, status));
        }
        if let Some(before) = self
            .baseline
            .as_ref()
            .and_then(|baseline| baseline.before_state(path))
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

    fn drain(&mut self) -> bool {
        let (events, gap) = self.hub.events_after(self.cursor);
        let updated = (gap && !self.overflowed) || !events.is_empty();
        self.overflowed |= gap;
        for event in events {
            self.cursor = self.cursor.max(event.sequence);
            self.record(event.path, event.status);
        }
        updated
    }

    fn record(&mut self, path: PathBuf, next: ToolFileChangeStatus) {
        if !path.starts_with(self.hub.root()) {
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

#[cfg(test)]
#[path = "tool_bash_changes_tests.rs"]
mod tests;
