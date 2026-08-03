#[cfg(any(unix, test))]
use std::path::Path;
use std::path::PathBuf;

#[cfg(any(unix, test))]
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

// Les parents larges restent exclus : ces dossiers servent uniquement à
// initialiser les gestionnaires d'outils et les configurations shell usuelles.
#[cfg(any(unix, test))]
const PROFILE_READ_DIRS: [&str; 12] = [
    ".nvm",
    ".local/bin",
    ".local/share/mise",
    ".rbenv",
    ".pyenv",
    ".asdf",
    ".volta",
    ".bun",
    ".cargo/bin",
    ".oh-my-zsh",
    ".zsh",
    ".config/zsh",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Mode {
    Workspace,
    ProfileCapture,
}

pub(super) struct Scope {
    pub mode: Mode,
    pub roots: Vec<PathBuf>,
    #[cfg(unix)]
    pub read_dirs: Vec<PathBuf>,
    #[cfg(unix)]
    pub read_files: Vec<PathBuf>,
}

impl Scope {
    pub fn workspace(roots: Vec<PathBuf>) -> Self {
        Self {
            mode: Mode::Workspace,
            roots,
            #[cfg(unix)]
            read_dirs: Vec::new(),
            #[cfg(unix)]
            read_files: Vec::new(),
        }
    }

    pub fn profile_capture(roots: Vec<PathBuf>) -> Self {
        #[cfg(unix)]
        let (read_dirs, read_files) = profile_roots();
        Self {
            mode: Mode::ProfileCapture,
            roots,
            #[cfg(unix)]
            read_dirs,
            #[cfg(unix)]
            read_files,
        }
    }
}

#[cfg(unix)]
fn profile_roots() -> (Vec<PathBuf>, Vec<PathBuf>) {
    dirs::home_dir()
        .and_then(|home| dunce::canonicalize(home).ok())
        .filter(|home| home.is_dir())
        .map(|home| (profile_dirs_in(&home), profile_files_in(&home)))
        .unwrap_or_default()
}

#[cfg(any(unix, test))]
fn profile_dirs_in(home: &Path) -> Vec<PathBuf> {
    PROFILE_READ_DIRS
        .iter()
        .filter_map(|name| safe_profile_dir(home, &home.join(name)))
        .collect()
}

#[cfg(any(unix, test))]
fn profile_files_in(home: &Path) -> Vec<PathBuf> {
    PROFILE_FILES
        .iter()
        .filter_map(|name| {
            let candidate = home.join(name);
            let metadata = candidate.symlink_metadata().ok()?;
            if !metadata.is_file() || super::tool_roots_path::has_symlink_below(home, &candidate) {
                return None;
            }
            let canonical = dunce::canonicalize(candidate).ok()?;
            canonical.starts_with(home).then_some(canonical)
        })
        .collect()
}

#[cfg(any(unix, test))]
fn safe_profile_dir(home: &Path, candidate: &Path) -> Option<PathBuf> {
    if super::tool_roots_path::has_symlink_below(home, candidate) {
        return None;
    }
    let canonical = super::tool_roots_path::canonical_dir(candidate)?;
    (canonical.starts_with(home) && canonical != home).then_some(canonical)
}

#[cfg(test)]
#[path = "scope_tests.rs"]
mod tests;
