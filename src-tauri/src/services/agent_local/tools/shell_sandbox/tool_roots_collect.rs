use super::tool_roots::{ToolRoots, MAX_READ_ROOTS, MAX_WRITE_ROOTS};
use super::tool_roots_entries::{
    push_read_dir, push_read_file, push_write_dir, push_write_file,
};
use super::tool_roots_path::{canonical_dir, contains_executable, is_tool_directory};
use std::path::{Path, PathBuf};

const USER_TOOL_READ_DIRS: [&str; 6] = [
    ".local/lib",
    ".local/share/pipx",
    ".local/share/uv",
    ".local/share/mise",
    ".local/share/pnpm",
    ".local/share/virtualenvs",
];

#[cfg(test)]
pub(super) fn collect_from(
    workspace_roots: &[PathBuf],
    platform_read_dirs: &[PathBuf],
    package_prefixes: &[PathBuf],
    home: Option<&Path>,
    path_inputs: &[PathBuf],
    path_overflow: bool,
    writable_cache_dirs: &[PathBuf],
) -> ToolRoots {
    let mut roots = ToolRoots::default();
    collect_into(
        &mut roots,
        workspace_roots,
        platform_read_dirs,
        package_prefixes,
        home,
        path_inputs,
        path_overflow,
        writable_cache_dirs,
    );
    roots
}

#[expect(clippy::too_many_arguments, reason = "boundary parameters remain explicit and locally audited")]
pub(super) fn collect_into(
    roots: &mut ToolRoots,
    workspace_roots: &[PathBuf],
    platform_read_dirs: &[PathBuf],
    package_prefixes: &[PathBuf],
    home: Option<&Path>,
    path_inputs: &[PathBuf],
    path_overflow: bool,
    writable_cache_dirs: &[PathBuf],
) {
    let input_home = home;
    let home = home.and_then(canonical_dir);

    for path in platform_read_dirs.iter().chain(package_prefixes) {
        push_read_dir(roots, path, workspace_roots, home.as_deref());
    }
    if let Some(home) = home.as_deref() {
        // Les proxies rustup du PATH chargent les compilateurs depuis ce dossier.
        push_read_dir(
            roots,
            &home.join(".rustup/toolchains"),
            workspace_roots,
            Some(home),
        );
        push_read_file(
            roots,
            &home.join(".gitconfig"),
            workspace_roots,
            home,
        );
        push_read_dir(roots, &home.join(".config/git"), workspace_roots, Some(home));
        push_read_file(
            roots,
            &home.join(".gitignore_global"),
            workspace_roots,
            home,
        );
        push_read_file(roots, &home.join(".npmrc"), workspace_roots, home);
        for relative in USER_TOOL_READ_DIRS {
            push_read_dir(
                roots,
                &home.join(relative),
                workspace_roots,
                Some(home),
            );
        }
    }

    let canonical_path_dirs = path_inputs
        .iter()
        .filter_map(|path| canonical_dir(path))
        .filter(|path| contains_executable(path))
        .collect::<Vec<_>>();
    for path in &canonical_path_dirs {
        push_read_dir(roots, path, workspace_roots, home.as_deref());
        if is_tool_directory(path) {
            if let Some(parent) = path.parent() {
                // Le dossier parent contient les bibliothèques de la même toolchain.
                let is_local_root = home
                    .as_deref()
                    .is_some_and(|home| parent == home.join(".local"));
                if !is_local_root {
                    push_read_dir(roots, parent, workspace_roots, home.as_deref());
                }
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
                roots,
                &path,
                workspace_roots,
                &canonical_path_dirs,
                home,
            );
        }
        push_write_file(
            roots,
            &home.join(".rustup/settings.toml"),
            workspace_roots,
            &canonical_path_dirs,
            home,
        );
    }
    debug_assert!(roots.read_dirs.len() + roots.read_files.len() <= MAX_READ_ROOTS);
    debug_assert!(roots.write_dirs.len() + roots.write_files.len() <= MAX_WRITE_ROOTS);
}
