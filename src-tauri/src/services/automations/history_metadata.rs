use super::HistoryEntry;
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

use super::history_store::{MAX_LINES, MAX_LINE_BYTES};

const MAX_METADATA_PATHS: usize = 16;

#[derive(Eq, Hash, PartialEq)]
enum OccurrenceKey {
    Run(uuid::Uuid),
    Legacy(String, String),
}

fn key(entry: &HistoryEntry) -> OccurrenceKey {
    entry.run_id.map_or_else(
        || OccurrenceKey::Legacy(entry.automation_id.clone(), entry.scheduled_for.clone()),
        OccurrenceKey::Run,
    )
}

pub(super) struct Metadata {
    byte_len: usize,
    line_count: usize,
    occurrences: HashSet<OccurrenceKey>,
}

impl Metadata {
    pub(super) fn from_content(content: &str) -> Self {
        let lines = content.lines().rev().take(MAX_LINES).collect::<Vec<_>>();
        let occurrences = lines
            .iter()
            .filter(|line| line.len() <= MAX_LINE_BYTES)
            .filter_map(|line| serde_json::from_str::<HistoryEntry>(line).ok())
            .map(|entry| key(&entry))
            .collect();
        Self {
            byte_len: content.len(),
            line_count: lines.len(),
            occurrences,
        }
    }

    pub(super) fn contains(&self, entry: &HistoryEntry) -> bool {
        self.occurrences.contains(&key(entry))
    }

    pub(super) fn byte_len(&self) -> usize {
        self.byte_len
    }

    pub(super) fn needs_rotation(&self, new_bytes: usize, max_bytes: usize) -> bool {
        self.line_count >= MAX_LINES || self.byte_len.saturating_add(new_bytes) > max_bytes
    }

    pub(super) fn record(&mut self, entry: &HistoryEntry, bytes: usize) {
        self.byte_len = self.byte_len.saturating_add(bytes);
        self.line_count = self.line_count.saturating_add(1).min(MAX_LINES);
        self.occurrences.insert(key(entry));
    }
}

#[derive(Default)]
pub(super) struct State(VecDeque<(PathBuf, Metadata)>);

impl State {
    pub(super) fn position(&self, path: &Path) -> Option<usize> {
        self.0.iter().position(|(known, _)| known == path)
    }

    pub(super) fn insert(&mut self, path: PathBuf, metadata: Metadata) -> usize {
        if self.0.len() == MAX_METADATA_PATHS {
            self.0.pop_front();
        }
        self.0.push_back((path, metadata));
        self.0.len() - 1
    }

    pub(super) fn get(&self, position: usize) -> &Metadata {
        &self.0[position].1
    }

    pub(super) fn get_mut(&mut self, position: usize) -> &mut Metadata {
        &mut self.0[position].1
    }
}
