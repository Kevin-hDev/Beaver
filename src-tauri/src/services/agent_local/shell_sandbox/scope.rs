use std::path::{Path, PathBuf};

const PROFILE_FILES: [&str; 10] = [
    ".zshenv",
    ".zprofile",
    ".zshrc",
    ".zlogin",
    ".bash_profile",
    ".bash_login",
    ".bashrc",
    ".profile",
    ".kshrc",
    ".cargo/env",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Mode {
    Workspace,
    ProfileCapture,
}

pub(super) struct Scope {
    pub mode: Mode,
    pub roots: Vec<PathBuf>,
    pub read_files: Vec<PathBuf>,
}

impl Scope {
    pub fn workspace(roots: Vec<PathBuf>) -> Self {
        Self {
            mode: Mode::Workspace,
            roots,
            read_files: Vec::new(),
        }
    }

    pub fn profile_capture(roots: Vec<PathBuf>) -> Self {
        Self {
            mode: Mode::ProfileCapture,
            roots,
            read_files: profile_files(),
        }
    }
}

fn profile_files() -> Vec<PathBuf> {
    dirs::home_dir()
        .and_then(|home| dunce::canonicalize(home).ok())
        .filter(|home| home.is_dir())
        .map(|home| profile_files_in(&home))
        .unwrap_or_default()
}

fn profile_files_in(home: &Path) -> Vec<PathBuf> {
    PROFILE_FILES
        .iter()
        .filter_map(|name| {
            let candidate = home.join(name);
            let metadata = candidate.symlink_metadata().ok()?;
            if !metadata.is_file()
                || super::tool_roots_path::has_symlink_below(home, &candidate)
            {
                return None;
            }
            let canonical = dunce::canonicalize(candidate).ok()?;
            canonical.starts_with(home).then_some(canonical)
        })
        .collect()
}

#[cfg(test)]
#[path = "scope_tests.rs"]
mod tests;
