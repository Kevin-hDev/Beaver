use super::types_tools::{ToolFileChange, ToolFileChangeStatus};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub const MAX_FILE_CHANGES: usize = 500;
pub const MAX_DIFF_FILE_BYTES: u64 = 1024 * 1024;
pub const MAX_FILE_CHANGE_DIFF_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FileSignature {
    len: u64,
    modified_ms: u128,
}

#[derive(Clone)]
pub struct FileState {
    signature: FileSignature,
    content: Option<Vec<u8>>,
}

pub fn capture(path: &Path, remaining_bytes: &mut usize) -> Option<FileState> {
    let metadata = std::fs::symlink_metadata(path).ok()?;
    if !metadata.file_type().is_file() {
        return None;
    }
    let modified_ms = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let content = capture_content(path, metadata.len(), remaining_bytes);
    Some(FileState {
        signature: FileSignature {
            len: metadata.len(),
            modified_ms,
        },
        content,
    })
}

pub fn direct_snapshot(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
) -> Option<(PathBuf, Option<FileState>)> {
    let key = match tool_name {
        "write_file" | "edit_file" => "path",
        _ => return None,
    };
    let raw = args.get(key)?.as_str()?;
    let candidate = if Path::new(raw).is_absolute() {
        PathBuf::from(raw)
    } else {
        working_dir.join(raw)
    };
    let validated = super::security::validate_write_path(&candidate).ok()?;
    let mut remaining = MAX_DIFF_FILE_BYTES as usize;
    let state = capture(&validated, &mut remaining);
    Some((validated, state))
}

pub fn direct_change(snapshot: (PathBuf, Option<FileState>)) -> Option<ToolFileChange> {
    let (path, before) = snapshot;
    let mut remaining = MAX_DIFF_FILE_BYTES as usize;
    let after = capture(&path, &mut remaining);
    build_change(&path, before.as_ref(), after.as_ref())
}

pub fn build_change(
    path: &Path,
    before: Option<&FileState>,
    after: Option<&FileState>,
) -> Option<ToolFileChange> {
    if states_equal(before, after) {
        return None;
    }
    let status = match (before, after) {
        (None, Some(_)) => ToolFileChangeStatus::Added,
        (Some(_), None) => ToolFileChangeStatus::Deleted,
        (Some(_), Some(_)) => ToolFileChangeStatus::Modified,
        (None, None) => return None,
    };
    let diff = build_diff(path, before, after);
    let (additions, deletions) = diff.as_ref().map_or((0, 0), diff_stats);
    Some(ToolFileChange {
        path: path.to_string_lossy().to_string(),
        status,
        additions,
        deletions,
        diff,
    })
}

fn capture_content(path: &Path, len: u64, remaining_bytes: &mut usize) -> Option<Vec<u8>> {
    if len > MAX_DIFF_FILE_BYTES
        || len as usize > *remaining_bytes
        || super::sensitive_data::is_sensitive_path(path)
    {
        return None;
    }
    let content = std::fs::read(path).ok()?;
    *remaining_bytes = remaining_bytes.saturating_sub(content.len());
    Some(content)
}

fn states_equal(before: Option<&FileState>, after: Option<&FileState>) -> bool {
    match (before, after) {
        (None, None) => true,
        (Some(before), Some(after)) => match (&before.content, &after.content) {
            (Some(before), Some(after)) => before == after,
            _ => before.signature == after.signature,
        },
        _ => false,
    }
}

fn build_diff(
    path: &Path,
    before: Option<&FileState>,
    after: Option<&FileState>,
) -> Option<crate::services::git::diff_preview::GitDiffPreview> {
    let old = state_content(before)?;
    let new = state_content(after)?;
    let old = redact_if_text(old);
    let new = redact_if_text(new);
    crate::services::git::diff_preview::preview_buffers(&old, &new, path).ok()
}

fn state_content(state: Option<&FileState>) -> Option<Vec<u8>> {
    match state {
        Some(state) => state.content.clone(),
        None => Some(Vec::new()),
    }
}

fn redact_if_text(content: Vec<u8>) -> Vec<u8> {
    String::from_utf8(content.clone())
        .map(|text| super::sensitive_data::redact_text(&text).into_bytes())
        .unwrap_or(content)
}

fn diff_stats(diff: &crate::services::git::diff_preview::GitDiffPreview) -> (usize, usize) {
    diff.hunks
        .iter()
        .flat_map(|hunk| &hunk.lines)
        .fold((0, 0), |(added, deleted), line| match line.kind.as_str() {
            "added" => (added + 1, deleted),
            "deleted" => (added, deleted + 1),
            _ => (added, deleted),
        })
}

#[cfg(test)]
#[path = "tool_file_changes_tests.rs"]
mod tests;
