use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use super::tool_file_changes::{capture, states_differ, FileState, MAX_FILE_CHANGE_DIFF_BYTES};
use super::types_tools::ToolFileChangeStatus;

const MAX_BASELINE_PATHS: usize = 500;
const MAX_BASELINE_DIRECTORIES: usize = 4_096;

pub struct DirectoryBaseline {
    root: PathBuf,
    initial: BTreeMap<PathBuf, EntryState>,
    incomplete: bool,
}

enum EntryState {
    File(FileState),
    Directory,
}

impl DirectoryBaseline {
    pub fn capture(root: &Path) -> Self {
        let (initial, incomplete) = scan(root);
        Self {
            root: root.to_path_buf(),
            initial,
            incomplete,
        }
    }

    pub fn is_incomplete(&self) -> bool {
        self.incomplete
    }

    pub fn before_state(&self, path: &Path) -> Option<Option<FileState>> {
        if !path.starts_with(&self.root) {
            return None;
        }
        Some(match self.initial.get(path) {
            Some(EntryState::File(state)) => Some(state.clone()),
            _ => None,
        })
    }

    pub fn current_paths(&self) -> (Vec<(PathBuf, ToolFileChangeStatus)>, bool) {
        let (current, current_incomplete) = scan(&self.root);
        let mut incomplete = self.incomplete || current_incomplete;
        let paths = self
            .initial
            .keys()
            .chain(current.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut changes = Vec::new();
        for path in paths {
            let status = status_for_change(
                self.initial.get(&path),
                current.get(&path),
                !self.incomplete,
                !current_incomplete,
            );
            let Some(status) = status else {
                continue;
            };
            if changes.len() >= MAX_BASELINE_PATHS {
                incomplete = true;
                break;
            }
            changes.push((path, status));
        }
        (changes, incomplete)
    }
}

fn scan(root: &Path) -> (BTreeMap<PathBuf, EntryState>, bool) {
    let mut entries = BTreeMap::new();
    let mut pending = VecDeque::from([root.to_path_buf()]);
    let mut remaining_bytes = MAX_FILE_CHANGE_DIFF_BYTES;
    let mut visited_directories = 0;
    let mut incomplete = false;
    'scan: while let Some(directory) = pending.pop_front() {
        if visited_directories >= MAX_BASELINE_DIRECTORIES {
            incomplete = true;
            break;
        }
        visited_directories += 1;
        let read_dir = match std::fs::read_dir(&directory) {
            Ok(read_dir) => read_dir,
            Err(_) => {
                incomplete = true;
                continue;
            }
        };
        for entry in read_dir {
            let Ok(entry) = entry else {
                incomplete = true;
                continue;
            };
            if entries.len() >= MAX_BASELINE_PATHS {
                incomplete = true;
                break 'scan;
            }
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                incomplete = true;
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                if super::tool_bash_change_hub::is_trackable(root, &path) {
                    entries.insert(path.clone(), EntryState::Directory);
                    if pending.len().saturating_add(visited_directories)
                        >= MAX_BASELINE_DIRECTORIES
                    {
                        incomplete = true;
                    } else {
                        pending.push_back(path);
                    }
                }
            } else if file_type.is_file() {
                match capture(&path, &mut remaining_bytes) {
                    Some(state) => {
                        entries.insert(path, EntryState::File(state));
                    }
                    None => incomplete = true,
                }
            }
        }
    }
    (entries, incomplete)
}

fn status_for_change(
    before: Option<&EntryState>,
    after: Option<&EntryState>,
    initial_complete: bool,
    current_complete: bool,
) -> Option<ToolFileChangeStatus> {
    match (before, after) {
        (None, Some(_)) if initial_complete => Some(ToolFileChangeStatus::Added),
        (Some(_), None) if current_complete => Some(ToolFileChangeStatus::Deleted),
        (Some(EntryState::Directory), Some(EntryState::Directory)) => None,
        (Some(EntryState::File(before)), Some(EntryState::File(after))) => {
            states_differ(Some(before), Some(after)).then_some(ToolFileChangeStatus::Modified)
        }
        (Some(_), Some(_)) => Some(ToolFileChangeStatus::Modified),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{status_for_change, DirectoryBaseline, EntryState};

    #[test]
    fn detects_net_file_and_directory_changes() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("old.txt"), "before").expect("initial file");
        let baseline = DirectoryBaseline::capture(root.path());
        std::fs::write(root.path().join("old.txt"), "after").expect("updated file");
        std::fs::create_dir(root.path().join("new-dir")).expect("new directory");

        let (changes, incomplete) = baseline.current_paths();

        assert!(!incomplete);
        assert_eq!(changes.len(), 2);
    }

    #[test]
    fn incomplete_scans_never_invent_additions_or_deletions() {
        let entry = EntryState::Directory;

        assert!(status_for_change(Some(&entry), None, true, false).is_none());
        assert!(status_for_change(None, Some(&entry), false, true).is_none());
    }
}
