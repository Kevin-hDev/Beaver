use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::tool_file_changes::{FileState, MAX_FILE_CHANGES, MAX_FILE_CHANGE_DIFF_BYTES};
use super::types_tools::ToolFileChange;

const MAX_SCAN_FILES: usize = 6_000;
const MAX_SCAN_DEPTH: usize = 8;
const MAX_SNAPSHOT_CONTENT_BYTES: usize = 32 * 1024 * 1024;
const SKIPPED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".turbo",
    ".cache",
];

#[derive(Default)]
pub struct FileSnapshot {
    files: BTreeMap<PathBuf, FileState>,
    complete: bool,
}

pub fn snapshot(root: &Path) -> FileSnapshot {
    let Ok(root) = root.canonicalize() else {
        return FileSnapshot::default();
    };
    let mut snapshot = FileSnapshot {
        files: BTreeMap::new(),
        complete: true,
    };
    let mut remaining_bytes = MAX_SNAPSHOT_CONTENT_BYTES;
    scan_dir(&root, 0, &mut snapshot, &mut remaining_bytes);
    snapshot
}

#[cfg(test)]
pub fn changed_paths(before: &FileSnapshot, after: &FileSnapshot) -> Vec<String> {
    changes(before, after)
        .into_iter()
        .map(|change| change.path)
        .collect()
}

pub fn changes(before: &FileSnapshot, after: &FileSnapshot) -> Vec<ToolFileChange> {
    if !before.complete || !after.complete {
        return Vec::new();
    }

    let mut remaining_diff_bytes = MAX_FILE_CHANGE_DIFF_BYTES;
    before
        .files
        .keys()
        .chain(after.files.keys())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .filter_map(|path| {
            let mut change = super::tool_file_changes::build_change(
                path,
                before.files.get(path),
                after.files.get(path),
            )?;
            if let Some(diff) = &change.diff {
                let size = crate::services::git::diff_preview::preview_content_bytes(diff);
                if size > remaining_diff_bytes {
                    change.diff = None;
                } else {
                    remaining_diff_bytes -= size;
                }
            }
            Some(change)
        })
        .take(MAX_FILE_CHANGES)
        .collect()
}

fn scan_dir(root: &Path, depth: usize, snapshot: &mut FileSnapshot, remaining_bytes: &mut usize) {
    if depth > MAX_SCAN_DEPTH || snapshot.files.len() >= MAX_SCAN_FILES {
        snapshot.complete = false;
        return;
    }

    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        if snapshot.files.len() >= MAX_SCAN_FILES {
            snapshot.complete = false;
            return;
        }
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            scan_dir(&path, depth + 1, snapshot, remaining_bytes);
        } else if file_type.is_file() {
            record_file(&path, snapshot, remaining_bytes);
        }
    }
}

fn should_skip_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| SKIPPED_DIRS.contains(&name))
        .unwrap_or(false)
}

fn record_file(path: &Path, snapshot: &mut FileSnapshot, remaining_bytes: &mut usize) {
    if let Some(state) = super::tool_file_changes::capture(path, remaining_bytes) {
        snapshot.files.insert(path.to_path_buf(), state);
    }
}

#[cfg(test)]
mod tests {
    use super::{changed_paths, snapshot};

    #[test]
    fn detects_created_files_under_root() {
        let dir = tempfile::tempdir().expect("tempdir");
        let before = snapshot(dir.path());

        let created = dir.path().join("created.md");
        std::fs::write(&created, "hello").expect("write");
        let after = snapshot(dir.path());
        let expected = created.canonicalize().expect("canonicalize");

        assert_eq!(
            changed_paths(&before, &after),
            vec![expected.to_string_lossy().to_string()]
        );
    }

    #[test]
    fn detects_deleted_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let deleted = dir.path().join("deleted.md");
        std::fs::write(&deleted, "hello").expect("write");
        let before = snapshot(dir.path());

        std::fs::remove_file(&deleted).expect("remove");
        let after = snapshot(dir.path());
        let expected = dir
            .path()
            .canonicalize()
            .expect("canonical root")
            .join("deleted.md");

        assert_eq!(
            changed_paths(&before, &after),
            vec![expected.to_string_lossy().to_string()]
        );
    }
}
