use std::path::{Component, Path, PathBuf};

pub(super) const MAX_READ_ROOTS: usize = 64;
pub(super) const MAX_WRITE_ROOTS: usize = 5;
const MAX_PATH_INPUTS: usize = 256;

#[derive(Default)]
pub(super) struct ToolRoots {
    pub read_dirs: Vec<PathBuf>,
    pub read_files: Vec<PathBuf>,
    pub write_dirs: Vec<PathBuf>,
    pub write_files: Vec<PathBuf>,
}

pub(super) fn collect(
    workspace_roots: &[PathBuf],
    platform_read_dirs: &[&str],
    package_prefixes: &[&str],
    executable: Option<&Path>,
) -> ToolRoots {
    let mut path_inputs = Vec::with_capacity(MAX_PATH_INPUTS + 1);
    if let Some(parent) = executable.and_then(Path::parent) {
        path_inputs.push(parent.to_path_buf());
    }
    let path_env = std::env::var_os("PATH").unwrap_or_default();
    path_inputs.extend(
        std::env::split_paths(&path_env)
            .take(MAX_PATH_INPUTS + 1 - path_inputs.len()),
    );
    let path_overflow = path_inputs.len() > MAX_PATH_INPUTS;
    path_inputs.truncate(MAX_PATH_INPUTS);
    let platform = platform_read_dirs.iter().map(PathBuf::from).collect::<Vec<_>>();
    let packages = package_prefixes.iter().map(PathBuf::from).collect::<Vec<_>>();
    collect_from(
        workspace_roots,
        &platform,
        &packages,
        dirs::home_dir().as_deref(),
        &path_inputs,
        path_overflow,
    )
}

fn collect_from(
    workspace_roots: &[PathBuf],
    platform_read_dirs: &[PathBuf],
    package_prefixes: &[PathBuf],
    home: Option<&Path>,
    path_inputs: &[PathBuf],
    path_overflow: bool,
) -> ToolRoots {
    let home = home.and_then(canonical_dir);
    let mut roots = ToolRoots::default();

    for path in platform_read_dirs.iter().chain(package_prefixes) {
        push_read_dir(&mut roots, path, workspace_roots, home.as_deref());
    }
    if let Some(home) = home.as_deref() {
        // Les proxies rustup du PATH chargent les compilateurs depuis ce dossier.
        push_read_dir(
            &mut roots,
            &home.join(".rustup/toolchains"),
            workspace_roots,
            Some(home),
        );
        push_read_file(&mut roots, &home.join(".gitconfig"), workspace_roots);
        push_read_dir(&mut roots, &home.join(".config/git"), workspace_roots, Some(home));
        push_read_file(&mut roots, &home.join(".gitignore_global"), workspace_roots);
        push_read_file(&mut roots, &home.join(".npmrc"), workspace_roots);
    }

    let canonical_path_dirs = path_inputs
        .iter()
        .filter_map(|path| canonical_dir(path))
        .collect::<Vec<_>>();
    for path in &canonical_path_dirs {
        push_read_dir(&mut roots, path, workspace_roots, home.as_deref());
        if is_tool_directory(path) {
            if let Some(parent) = path.parent() {
                push_read_dir(&mut roots, parent, workspace_roots, home.as_deref());
            }
        }
    }

    if let Some(home) = home.as_deref().filter(|_| !path_overflow) {
        for path in [
            home.join(".cargo/registry"),
            home.join(".cargo/git"),
            home.join(".npm/_cacache"),
            home.join(".cache"),
        ] {
            push_write_dir(&mut roots, &path, workspace_roots, &canonical_path_dirs, home);
        }
        push_write_file(
            &mut roots,
            &home.join(".rustup/settings.toml"),
            workspace_roots,
            &canonical_path_dirs,
        );
    }
    roots
}

fn push_read_dir(
    roots: &mut ToolRoots,
    path: &Path,
    workspace_roots: &[PathBuf],
    home: Option<&Path>,
) {
    if read_len(roots) >= MAX_READ_ROOTS {
        return;
    }
    let Some(path) = canonical_dir(path) else {
        return;
    };
    if forbidden_broad_root(&path, home)
        || overlaps_workspace(&path, workspace_roots)
        || roots.read_dirs.contains(&path)
    {
        return;
    }
    roots.read_dirs.push(path);
}

fn push_read_file(roots: &mut ToolRoots, path: &Path, workspace_roots: &[PathBuf]) {
    if read_len(roots) >= MAX_READ_ROOTS {
        return;
    }
    let Some(path) = canonical_file(path) else { return };
    if overlaps_workspace(&path, workspace_roots) || roots.read_files.contains(&path) {
        return;
    }
    roots.read_files.push(path);
}

fn push_write_dir(
    roots: &mut ToolRoots,
    path: &Path,
    workspace_roots: &[PathBuf],
    path_dirs: &[PathBuf],
    home: &Path,
) {
    if write_len(roots) >= MAX_WRITE_ROOTS {
        return;
    }
    let Some(path) = canonical_write_dir(path) else {
        return;
    };
    if forbidden_broad_root(&path, Some(home))
        || overlaps_workspace(&path, workspace_roots)
        || path_dirs.iter().any(|bin| bin.starts_with(&path))
        || roots.write_dirs.contains(&path)
    {
        return;
    }
    roots.write_dirs.push(path);
}

fn push_write_file(
    roots: &mut ToolRoots,
    path: &Path,
    workspace_roots: &[PathBuf],
    path_dirs: &[PathBuf],
) {
    if write_len(roots) >= MAX_WRITE_ROOTS {
        return;
    }
    let Some(path) = canonical_write_file(path) else {
        return;
    };
    if overlaps_workspace(&path, workspace_roots)
        || path_dirs
            .iter()
            .any(|bin| bin.starts_with(&path) || path.starts_with(bin))
        || roots.write_files.contains(&path)
    {
        return;
    }
    roots.write_files.push(path);
}

fn canonical_dir(path: &Path) -> Option<PathBuf> {
    valid_input(path)
        .then(|| dunce::canonicalize(path).ok())
        .flatten()
        .filter(|path| path.is_dir())
}

fn canonical_file(path: &Path) -> Option<PathBuf> {
    valid_input(path)
        .then(|| dunce::canonicalize(path).ok())
        .flatten()
        .filter(|path| path.is_file())
}

fn canonical_write_dir(path: &Path) -> Option<PathBuf> {
    (!path.symlink_metadata().ok()?.file_type().is_symlink())
        .then(|| canonical_dir(path))
        .flatten()
}

fn canonical_write_file(path: &Path) -> Option<PathBuf> {
    (!path.symlink_metadata().ok()?.file_type().is_symlink())
        .then(|| canonical_file(path))
        .flatten()
}

fn valid_input(path: &Path) -> bool {
    path.is_absolute()
        && path.as_os_str().to_string_lossy().chars().count()
            <= super::super::directory_access::MAX_PATH_CHARS
        && !path.components().any(|part| matches!(part, Component::ParentDir))
}

fn overlaps_workspace(path: &Path, workspace_roots: &[PathBuf]) -> bool {
    workspace_roots
        .iter()
        .any(|root| path.starts_with(root) || root.starts_with(path))
}

fn forbidden_broad_root(path: &Path, home: Option<&Path>) -> bool {
    path.parent().is_none()
        || home.is_some_and(|home| path == home || home.starts_with(path))
}

fn is_tool_directory(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        let name = name.to_string_lossy();
        name.eq_ignore_ascii_case("bin")
            || name.eq_ignore_ascii_case("sbin")
            || name.eq_ignore_ascii_case("shims")
    })
}

fn read_len(roots: &ToolRoots) -> usize {
    roots.read_dirs.len() + roots.read_files.len()
}

fn write_len(roots: &ToolRoots) -> usize {
    roots.write_dirs.len() + roots.write_files.len()
}

#[cfg(test)]
#[path = "tool_roots_tests.rs"]
mod tests;
