use super::blob_preview::validate_repo_path;
use super::diff_preview_serialize::serialize_patch;
use super::history::{find_reachable_commit, open_current_branch};
use git2::{Diff, DiffFindOptions, DiffOptions, Patch};
use std::path::Path;

pub use super::diff_preview_model::GitDiffPreview;

const MAX_DIFF_DELTAS: usize = 10_000;

pub fn preview_buffers(
    old_buffer: &[u8],
    new_buffer: &[u8],
    path: &Path,
) -> Result<GitDiffPreview, String> {
    let mut options = base_options();
    let patch = Patch::from_buffers(
        old_buffer,
        Some(path),
        new_buffer,
        Some(path),
        Some(&mut options),
    )
    .map_err(|_| unavailable())?;
    Ok(serialize_patch(&patch))
}

pub fn is_bounded_preview(preview: &GitDiffPreview) -> bool {
    if preview.hunks.len() > super::diff_preview_serialize::MAX_HUNKS {
        return false;
    }
    let lines = preview.hunks.iter().flat_map(|hunk| &hunk.lines);
    let mut line_count = 0usize;
    let mut total_bytes = 0usize;
    for line in lines {
        line_count += 1;
        total_bytes = total_bytes.saturating_add(line.content.len());
        if line_count > super::diff_preview_serialize::MAX_LINES
            || line.content.len() > super::diff_preview_serialize::MAX_LINE_BYTES
            || total_bytes > super::diff_preview_serialize::MAX_TOTAL_BYTES
        {
            return false;
        }
    }
    true
}

pub fn preview_content_bytes(preview: &GitDiffPreview) -> usize {
    preview
        .hunks
        .iter()
        .flat_map(|hunk| &hunk.lines)
        .fold(0usize, |total, line| {
            total.saturating_add(line.content.len())
        })
}

pub fn read_commit_diff(
    repo_path: &Path,
    expected_branch: &str,
    commit_id: &str,
    file_path: &str,
    previous_path: Option<&str>,
) -> Result<GitDiffPreview, String> {
    validate_paths(file_path, previous_path)?;
    let (repo, head) = open_current_branch(repo_path, expected_branch)?;
    let commit = find_reachable_commit(&repo, head, commit_id)?;
    let tree = commit.tree().map_err(|_| unavailable())?;
    let parent_tree = commit.parent(0).ok().and_then(|parent| parent.tree().ok());
    let mut options = if previous_path.is_some() {
        base_options()
    } else {
        scoped_options(file_path)
    };
    let mut diff = repo
        .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut options))
        .map_err(|_| unavailable())?;
    find_renames(&mut diff)?;
    build_preview(&diff, file_path, previous_path)
}

pub fn read_working_diff(
    repo_path: &Path,
    expected_branch: &str,
    expected_head: &str,
    file_path: &str,
    previous_path: Option<&str>,
) -> Result<GitDiffPreview, String> {
    validate_paths(file_path, previous_path)?;
    let (repo, head) = open_current_branch(repo_path, expected_branch)?;
    if find_reachable_commit(&repo, head, expected_head)?.id() != head {
        return Err(unavailable());
    }
    let tree = repo
        .find_commit(head)
        .and_then(|commit| commit.tree())
        .map_err(|_| unavailable())?;
    let mut options = if previous_path.is_some() {
        base_options()
    } else {
        scoped_options(file_path)
    };
    options
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .show_untracked_content(true);
    let mut diff = repo
        .diff_tree_to_workdir_with_index(Some(&tree), Some(&mut options))
        .map_err(|_| unavailable())?;
    find_renames(&mut diff)?;
    build_preview(&diff, file_path, previous_path)
}

fn scoped_options(file_path: &str) -> DiffOptions {
    let mut options = base_options();
    options.disable_pathspec_match(true).pathspec(file_path);
    options
}

fn base_options() -> DiffOptions {
    let mut options = DiffOptions::new();
    options.context_lines(3);
    options
}

fn find_renames(diff: &mut Diff<'_>) -> Result<(), String> {
    let mut find = DiffFindOptions::new();
    find.renames(true).copies(false).for_untracked(true);
    diff.find_similar(Some(&mut find))
        .map_err(|_| unavailable())
}

fn build_preview(
    diff: &Diff<'_>,
    file_path: &str,
    previous_path: Option<&str>,
) -> Result<GitDiffPreview, String> {
    let index = diff
        .deltas()
        .take(MAX_DIFF_DELTAS)
        .position(|delta| delta_matches(&delta, file_path, previous_path))
        .ok_or_else(unavailable)?;
    let Some(patch) = Patch::from_diff(diff, index).map_err(|_| unavailable())? else {
        let binary = diff
            .get_delta(index)
            .is_some_and(|delta| delta.flags().is_binary());
        return Ok(GitDiffPreview {
            hunks: Vec::new(),
            truncated: false,
            binary,
        });
    };
    Ok(serialize_patch(&patch))
}

fn delta_matches(
    delta: &git2::DiffDelta<'_>,
    file_path: &str,
    previous_path: Option<&str>,
) -> bool {
    let old_path = delta.old_file().path().and_then(Path::to_str);
    let new_path = delta.new_file().path().and_then(Path::to_str);
    let selected = old_path == Some(file_path) || new_path == Some(file_path);
    selected && previous_path.is_none_or(|path| old_path == Some(path))
}

fn validate_paths(file_path: &str, previous_path: Option<&str>) -> Result<(), String> {
    validate_repo_path(file_path)?;
    if let Some(path) = previous_path {
        validate_repo_path(path)?;
    }
    Ok(())
}

fn unavailable() -> String {
    "Aperçu Git indisponible".to_string()
}
