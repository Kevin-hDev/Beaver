use std::path::{Component, Path, PathBuf};

const MAX_EXECUTABLE_SCAN: usize = 4_096;

pub(super) fn canonical_dir(path: &Path) -> Option<PathBuf> {
    valid_input(path)
        .then(|| dunce::canonicalize(path).ok())
        .flatten()
        .filter(|path| path.is_dir())
}

pub(super) fn canonical_file(path: &Path) -> Option<PathBuf> {
    valid_input(path)
        .then(|| dunce::canonicalize(path).ok())
        .flatten()
        .filter(|path| path.is_file())
}

pub(super) fn canonical_write_dir(path: &Path) -> Option<PathBuf> {
    (!path.symlink_metadata().ok()?.file_type().is_symlink())
        .then(|| canonical_dir(path))
        .flatten()
}

pub(super) fn canonical_write_file(path: &Path) -> Option<PathBuf> {
    (!path.symlink_metadata().ok()?.file_type().is_symlink())
        .then(|| canonical_file(path))
        .flatten()
}

pub(super) fn overlaps_workspace(path: &Path, workspace_roots: &[PathBuf]) -> bool {
    workspace_roots
        .iter()
        .any(|root| path.starts_with(root) || root.starts_with(path))
}

pub(super) fn forbidden_broad_root(path: &Path, home: Option<&Path>) -> bool {
    path.parent().is_none()
        || home.is_some_and(|home| path == home || home.starts_with(path))
        || private_store().is_some_and(|private| private.starts_with(path))
}

fn private_store() -> Option<PathBuf> {
    let path = crate::services::paths::data_dir();
    dunce::canonicalize(&path).ok().or_else(|| path.is_absolute().then_some(path))
}

pub(super) fn has_symlink_below(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return true;
    };
    let mut current = root.to_path_buf();
    relative.components().any(|component| {
        current.push(component.as_os_str());
        current
            .symlink_metadata()
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
    })
}

pub(super) fn is_tool_directory(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        let name = name.to_string_lossy();
        name.eq_ignore_ascii_case("bin")
            || name.eq_ignore_ascii_case("sbin")
            || name.eq_ignore_ascii_case("shims")
    })
}

pub(super) fn contains_executable(path: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(path) else {
        return false;
    };
    entries.take(MAX_EXECUTABLE_SCAN).flatten().any(|entry| {
        let Ok(metadata) = entry.metadata() else {
            return false;
        };
        if !metadata.is_file() {
            return false;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode() & 0o111 != 0
        }
        #[cfg(windows)]
        {
            entry.path().extension().is_some_and(|extension| {
                matches!(
                    extension.to_string_lossy().to_ascii_lowercase().as_str(),
                    "exe" | "cmd" | "bat" | "com"
                )
            })
        }
        #[cfg(not(any(unix, windows)))]
        {
            false
        }
    })
}

fn valid_input(path: &Path) -> bool {
    path.is_absolute()
        && path.as_os_str().to_string_lossy().chars().count()
            <= super::super::directory_access::MAX_PATH_CHARS
        && !path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
}
