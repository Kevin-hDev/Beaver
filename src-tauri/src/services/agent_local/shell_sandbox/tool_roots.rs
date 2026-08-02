use super::tool_roots_entries::{push_read_dir, push_read_file, push_write_dir, push_write_file};
use super::tool_roots_path::{canonical_dir, is_tool_directory};
use std::path::{Path, PathBuf};

pub(super) const MAX_READ_ROOTS: usize = 64;
const MAX_PATH_INPUTS: usize = 256;
const WRITABLE_CACHE_DIRS: [&str; 7] = [
    ".cargo/registry",
    ".cargo/git",
    ".npm/_cacache",
    ".cache/pip",
    ".cache/uv",
    ".cache/go-build",
    ".cache/yarn",
];
// La borne suit la liste blanche ci-dessus, plus le fichier rustup autorisé.
pub(super) const MAX_WRITE_ROOTS: usize = WRITABLE_CACHE_DIRS.len() + 1;

#[derive(Default)]
pub(super) struct ToolRoots {
    pub read_dirs: Vec<PathBuf>,
    pub read_files: Vec<PathBuf>,
    pub write_dirs: Vec<PathBuf>,
    pub write_files: Vec<PathBuf>,
    pub read_limit_reached: bool,
    pub write_limit_reached: bool,
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
    let roots = collect_from(
        workspace_roots,
        &platform,
        &packages,
        dirs::home_dir().as_deref(),
        &path_inputs,
        path_overflow,
    );
    if path_overflow {
        eprintln!("[shell-sandbox] writable tool caches disabled: PATH entry limit exceeded");
    }
    if roots.read_limit_reached {
        eprintln!("[shell-sandbox] read-only tool root limit reached");
    }
    if roots.write_limit_reached {
        eprintln!("[shell-sandbox] writable tool root limit reached");
    }
    roots
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
        push_read_file(
            &mut roots,
            &home.join(".gitconfig"),
            workspace_roots,
            home,
        );
        push_read_dir(&mut roots, &home.join(".config/git"), workspace_roots, Some(home));
        push_read_file(
            &mut roots,
            &home.join(".gitignore_global"),
            workspace_roots,
            home,
        );
        push_read_file(&mut roots, &home.join(".npmrc"), workspace_roots, home);
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
        for relative in WRITABLE_CACHE_DIRS {
            push_write_dir(
                &mut roots,
                &home.join(relative),
                workspace_roots,
                &canonical_path_dirs,
                home,
            );
        }
        push_write_file(
            &mut roots,
            &home.join(".rustup/settings.toml"),
            workspace_roots,
            &canonical_path_dirs,
            home,
        );
    }
    roots
}

#[cfg(test)]
#[path = "tool_roots_tests.rs"]
mod tests;
