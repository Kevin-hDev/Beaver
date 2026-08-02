use super::tool_roots::{ToolRoots, MAX_READ_ROOTS, MAX_WRITE_ROOTS};
use super::tool_roots_path::{
    canonical_dir, canonical_file, canonical_write_dir, canonical_write_file,
    forbidden_broad_root, has_symlink_below, overlaps_workspace,
};
use std::path::{Path, PathBuf};

pub(super) fn push_read_dir(
    roots: &mut ToolRoots,
    path: &Path,
    workspace_roots: &[PathBuf],
    home: Option<&Path>,
) {
    let scoped_home = home.filter(|home| path.starts_with(home));
    if scoped_home.is_some_and(|home| has_symlink_below(home, path)) {
        return;
    }
    let Some(path) = canonical_dir(path) else { return };
    if scoped_home.is_some_and(|home| !path.starts_with(home))
        || forbidden_broad_root(&path, home)
        || overlaps_workspace(&path, workspace_roots)
        || roots.read_dirs.contains(&path)
    {
        return;
    }
    if read_len(roots) >= MAX_READ_ROOTS {
        roots.read_limit_reached = true;
        return;
    }
    roots.read_dirs.push(path);
}

pub(super) fn push_read_file(
    roots: &mut ToolRoots,
    path: &Path,
    workspace_roots: &[PathBuf],
    home: &Path,
) {
    if has_symlink_below(home, path) {
        return;
    }
    let Some(path) = canonical_file(path) else { return };
    if !path.starts_with(home)
        || overlaps_workspace(&path, workspace_roots)
        || roots.read_files.contains(&path)
    {
        return;
    }
    if read_len(roots) >= MAX_READ_ROOTS {
        roots.read_limit_reached = true;
        return;
    }
    roots.read_files.push(path);
}

pub(super) fn push_write_dir(
    roots: &mut ToolRoots,
    path: &Path,
    workspace_roots: &[PathBuf],
    path_dirs: &[PathBuf],
    home: &Path,
) {
    if has_symlink_below(home, path) {
        return;
    }
    let Some(path) = canonical_write_dir(path) else { return };
    if !path.starts_with(home)
        || forbidden_broad_root(&path, Some(home))
        || overlaps_workspace(&path, workspace_roots)
        || path_dirs.iter().any(|bin| bin.starts_with(&path))
        || roots.write_dirs.contains(&path)
    {
        return;
    }
    if write_len(roots) >= MAX_WRITE_ROOTS {
        roots.write_limit_reached = true;
        return;
    }
    roots.write_dirs.push(path);
}

pub(super) fn push_write_file(
    roots: &mut ToolRoots,
    path: &Path,
    workspace_roots: &[PathBuf],
    path_dirs: &[PathBuf],
    home: &Path,
) {
    if has_symlink_below(home, path) {
        return;
    }
    let Some(path) = canonical_write_file(path) else { return };
    if !path.starts_with(home)
        || overlaps_workspace(&path, workspace_roots)
        || path_dirs
            .iter()
            .any(|bin| bin.starts_with(&path) || path.starts_with(bin))
        || roots.write_files.contains(&path)
    {
        return;
    }
    if write_len(roots) >= MAX_WRITE_ROOTS {
        roots.write_limit_reached = true;
        return;
    }
    roots.write_files.push(path);
}

fn read_len(roots: &ToolRoots) -> usize {
    roots.read_dirs.len() + roots.read_files.len()
}

fn write_len(roots: &ToolRoots) -> usize {
    roots.write_dirs.len() + roots.write_files.len()
}
