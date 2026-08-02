use super::tool_cache_roots::CacheKind;
use super::tool_roots_path::has_symlink_below;
use std::path::{Component, Path, PathBuf};

const MAX_PATH_INPUTS: usize = 256;
const EXECUTABLE_SUFFIXES: [&str; 4] = ["", ".exe", ".cmd", ".bat"];

pub(super) fn ensure_defaults(
    kinds: &[CacheKind],
    selected: &[PathBuf],
    defaults: &[PathBuf],
    home: &Path,
) {
    let path_dirs = std::env::var_os("PATH")
        .map(|value| {
            std::env::split_paths(&value)
                .take(MAX_PATH_INPUTS)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut failed = false;
    for ((kind, selected), default) in kinds.iter().zip(selected).zip(defaults) {
        if selected == default
            && tool_available(*kind, &path_dirs)
            && !ensure_dir(home, selected)
        {
            failed = true;
        }
    }
    if failed {
        eprintln!("[shell-sandbox] one or more tool caches are unavailable");
    }
}

fn tool_available(kind: CacheKind, path_dirs: &[PathBuf]) -> bool {
    let names: &[&str] = match kind {
        CacheKind::CargoRegistry | CacheKind::CargoGit => &["cargo"],
        CacheKind::Npm => &["npm"],
        CacheKind::Pip => &["pip", "pip3"],
        CacheKind::Uv => &["uv"],
        CacheKind::GoBuild | CacheKind::GoModules | CacheKind::GoSumDb => &["go"],
        CacheKind::Yarn => &["yarn"],
    };
    path_dirs.iter().any(|directory| {
        names.iter().any(|name| {
            EXECUTABLE_SUFFIXES
                .iter()
                .any(|suffix| directory.join(format!("{name}{suffix}")).is_file())
        })
    })
}

fn ensure_dir(home: &Path, path: &Path) -> bool {
    let input_home = home;
    let Some(home) = dunce::canonicalize(home).ok().filter(|path| path.is_dir()) else {
        return false;
    };
    let path = path
        .strip_prefix(input_home)
        .map(|relative| home.join(relative))
        .unwrap_or_else(|_| path.to_path_buf());
    if !path.is_absolute()
        || !path.starts_with(&home)
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        || has_symlink_below(&home, &path)
        || std::fs::create_dir_all(&path).is_err()
        || has_symlink_below(&home, &path)
    {
        return false;
    }
    dunce::canonicalize(path).is_ok_and(|path| path.starts_with(home))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_an_exact_missing_cache_but_rejects_an_external_path() {
        let temp = tempfile::tempdir().expect("tempdir");
        let home = temp.path().join("home");
        let cache = home.join(".cache/tool");
        std::fs::create_dir(&home).expect("home");

        assert!(ensure_dir(&home, &cache));
        assert!(cache.is_dir());
        assert!(!ensure_dir(&home, &temp.path().join("external")));
    }

    #[cfg(unix)]
    #[test]
    fn refuses_a_cache_redirected_by_a_parent_symlink() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().expect("tempdir");
        let home = temp.path().join("home");
        let external = temp.path().join("external");
        std::fs::create_dir(&home).expect("home");
        std::fs::create_dir(&external).expect("external");
        symlink(&external, home.join(".cache")).expect("cache link");

        assert!(!ensure_dir(&home, &home.join(".cache/tool")));
        assert!(!external.join("tool").exists());
    }
}
