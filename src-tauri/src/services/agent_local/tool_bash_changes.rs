use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::tool_bash_change_hub::{workspace_hub, WorkspaceEventHub};
use super::types_tools::{ToolFileChange, ToolFileChangeStatus};

const MAX_TRACKED_PATHS: usize = 500;
const WATCHER_START_TIMEOUT_MS: u64 = 1_000;

pub struct ChangeTracker {
    hub: std::sync::Arc<WorkspaceEventHub>,
    cursor: u64,
    changes: BTreeMap<PathBuf, ToolFileChangeStatus>,
    overflowed: bool,
}

impl ChangeTracker {
    pub async fn start(root: &Path) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|_| "Suivi des fichiers indisponible.".to_string())?;
        let setup = tokio::task::spawn_blocking(move || workspace_hub(root));
        let hub = tokio::time::timeout(
            std::time::Duration::from_millis(WATCHER_START_TIMEOUT_MS),
            setup,
        )
        .await
        .map_err(|_| "Suivi des fichiers indisponible.".to_string())?
        .map_err(|_| "Suivi des fichiers indisponible.".to_string())??;
        let cursor = hub.sequence();
        Ok(Self {
            hub,
            cursor,
            changes: BTreeMap::new(),
            overflowed: false,
        })
    }

    #[cfg(test)]
    pub fn changes(&mut self) -> Vec<ToolFileChange> {
        self.drain();
        self.snapshot()
    }

    pub fn updated_changes(&mut self) -> Option<(Vec<ToolFileChange>, bool)> {
        if !self.drain() {
            return None;
        }
        Some((self.snapshot(), self.overflowed))
    }

    fn snapshot(&self) -> Vec<ToolFileChange> {
        self.changes
            .iter()
            .map(|(path, status)| ToolFileChange {
                path: path.to_string_lossy().to_string(),
                status: *status,
                additions: 0,
                deletions: 0,
                diff: None,
            })
            .collect()
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

#[cfg(test)]
#[path = "tool_bash_changes_tests.rs"]
mod tests;
