use crate::models::WakeupRun;
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

use super::{MAX_LINES, MAX_LOG_LINE_BYTES};

const MAX_METADATA_PATHS: usize = 16;

#[derive(Eq, Hash, PartialEq)]
struct OccurrenceKey {
    wakeup_id: String,
    scheduled_for: String,
}

impl OccurrenceKey {
    fn from_entry(entry: &WakeupRun) -> Self {
        Self {
            wakeup_id: entry.wakeup_id.clone(),
            scheduled_for: entry.scheduled_for.clone(),
        }
    }
}

pub(super) struct LogMetadata {
    byte_len: usize,
    line_count: usize,
    occurrences: HashSet<OccurrenceKey>,
}

impl LogMetadata {
    pub(super) fn from_content(content: &str) -> Self {
        let lines = content.lines().rev().take(MAX_LINES).collect::<Vec<_>>();
        let occurrences = lines
            .iter()
            .filter(|line| line.len() <= MAX_LOG_LINE_BYTES)
            .filter_map(|line| serde_json::from_str::<WakeupRun>(line).ok())
            .map(|run| OccurrenceKey::from_entry(&run))
            .collect();
        Self {
            byte_len: content.len(),
            line_count: lines.len(),
            occurrences,
        }
    }

    pub(super) fn contains(&self, entry: &WakeupRun) -> bool {
        self.occurrences.contains(&OccurrenceKey::from_entry(entry))
    }

    pub(super) fn byte_len(&self) -> usize {
        self.byte_len
    }

    pub(super) fn needs_rotation(&self, new_bytes: usize, max_bytes: usize) -> bool {
        self.line_count >= MAX_LINES || self.byte_len.saturating_add(new_bytes) > max_bytes
    }

    pub(super) fn record(&mut self, entry: &WakeupRun, bytes: usize) {
        self.byte_len = self.byte_len.saturating_add(bytes);
        self.line_count = self.line_count.saturating_add(1).min(MAX_LINES);
        if self.occurrences.len() < MAX_LINES {
            self.occurrences.insert(OccurrenceKey::from_entry(entry));
        }
    }
}

#[derive(Default)]
pub(super) struct LogStoreState {
    metadata: VecDeque<(PathBuf, LogMetadata)>,
}

impl LogStoreState {
    pub(super) fn position(&self, path: &Path) -> Option<usize> {
        self.metadata.iter().position(|(known, _)| known == path)
    }

    pub(super) fn insert(&mut self, path: PathBuf, metadata: LogMetadata) -> usize {
        if self.metadata.len() == MAX_METADATA_PATHS {
            self.metadata.pop_front();
        }
        self.metadata.push_back((path, metadata));
        self.metadata.len() - 1
    }

    pub(super) fn get(&self, position: usize) -> &LogMetadata {
        &self.metadata[position].1
    }

    pub(super) fn get_mut(&mut self, position: usize) -> &mut LogMetadata {
        &mut self.metadata[position].1
    }
}
