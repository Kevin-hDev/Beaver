use super::tool_roots_entries::{push_read_dir, push_read_file, push_write_dir, push_write_file};
use super::tool_roots_path::{canonical_dir, contains_executable, is_tool_directory};
use std::path::{Path, PathBuf};

pub(super) const MAX_READ_ROOTS: usize = 64;
pub(super) const MAX_WRITE_ROOTS: usize = super::tool_cache_roots::MAX_WRITE_DIRS + 1;

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
    collect_with_access(
        workspace_roots,
        platform_read_dirs,
        package_prefixes,
        executable,
        true,
    )
}

pub(super) fn collect_read_only(
    workspace_roots: &[PathBuf],
    platform_read_dirs: &[&str],
    package_prefixes: &[&str],
    executable: Option<&Path>,
) -> ToolRoots {
    collect_with_access(
        workspace_roots,
        platform_read_dirs,
        package_prefixes,
        executable,
        false,
    )
}

fn collect_with_access(
    workspace_roots: &[PathBuf],
    platform_read_dirs: &[&str],
    package_prefixes: &[&str],
    executable: Option<&Path>,
    allow_writes: bool,
) -> ToolRoots {
    let (configured_path, configured_overflow) = super::super::shell_environment::entries();
    let max_paths = super::super::shell_environment::MAX_PATH_INPUTS;
    let mut path_inputs = Vec::with_capacity(max_paths + 1);
    if let Some(parent) = executable.and_then(Path::parent) {
        path_inputs.push(parent.to_path_buf());
    }
    path_inputs.extend(
        configured_path
            .into_iter()
            .take(max_paths + 1 - path_inputs.len()),
    );
    let path_overflow = configured_overflow || path_inputs.len() > max_paths;
    path_inputs.truncate(max_paths);
    let platform = platform_read_dirs.iter().map(PathBuf::from).collect::<Vec<_>>();
    let packages = package_prefixes.iter().map(PathBuf::from).collect::<Vec<_>>();
    let home = dirs::home_dir().and_then(|path| canonical_dir(&path));
    let writable_cache_dirs = if allow_writes {
        home.as_deref()
            .map(|home| super::tool_cache_roots::collect(home, &path_inputs, path_overflow))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let roots = collect_from(
        workspace_roots,
        &platform,
        &packages,
        home.as_deref(),
        &path_inputs,
        path_overflow || !allow_writes,
        &writable_cache_dirs,
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
    writable_cache_dirs: &[PathBuf],
) -> ToolRoots {
    let input_home = home;
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
        .filter(|path| contains_executable(path))
        .collect::<Vec<_>>();
    for path in &canonical_path_dirs {
        push_read_dir(&mut roots, path, workspace_roots, home.as_deref());
        if is_tool_directory(path) {
            if let Some(parent) = path.parent() {
                // Le filtre d'exécutable porte sur le dossier PATH ; son parent
                // fournit les bibliothèques et ressources de la même toolchain.
                push_read_dir(&mut roots, parent, workspace_roots, home.as_deref());
            }
        }
    }

    if let Some(home) = home.as_deref().filter(|_| !path_overflow) {
        for path in writable_cache_dirs {
            let path = input_home
                .and_then(|input| path.strip_prefix(input).ok())
                .map(|relative| home.join(relative))
                .unwrap_or_else(|| path.clone());
            push_write_dir(
                &mut roots,
                &path,
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
