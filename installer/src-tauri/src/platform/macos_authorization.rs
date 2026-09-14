use crate::error::InstallerError;
use std::path::{Component, Path, PathBuf};

#[path = "macos_authorization_ffi.rs"]
mod ffi;
pub use ffi::AuthorizationSession;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProtectedTool {
    Ditto,
    Move,
    Remove,
}

impl ProtectedTool {
    pub fn path(self) -> &'static Path {
        Path::new(match self {
            Self::Ditto => "/usr/bin/ditto",
            Self::Move => "/bin/mv",
            Self::Remove => "/bin/rm",
        })
    }
}

#[derive(Debug, Clone)]
pub struct AuthorizationScope {
    install_root: PathBuf,
    source_root: PathBuf,
}

impl AuthorizationScope {
    pub fn new(install_root: &Path, source_root: &Path) -> Result<Self, InstallerError> {
        if install_root != Path::new("/Applications")
            || !safe_absolute(source_root)
            || source_root == Path::new("/Applications")
        {
            return Err(InstallerError::InstallFailed);
        }
        Ok(Self {
            install_root: install_root.to_path_buf(),
            source_root: source_root.to_path_buf(),
        })
    }
}

pub fn validate_call(
    tool: ProtectedTool,
    arguments: &[PathBuf],
    scope: &AuthorizationScope,
) -> Result<(), InstallerError> {
    if !system_tool_is_regular(tool.path()) {
        return Err(InstallerError::InstallFailed);
    }
    let valid = match tool {
        ProtectedTool::Remove => arguments.len() == 1 && allowed_sibling(&arguments[0], scope),
        ProtectedTool::Move => {
            arguments.len() == 2
                && allowed_sibling(&arguments[0], scope)
                && allowed_sibling(&arguments[1], scope)
        }
        ProtectedTool::Ditto => {
            arguments.len() == 2
                && arguments[0] == scope.source_root.join("Beaver.app")
                && allowed_sibling(&arguments[1], scope)
        }
    };
    valid.then_some(()).ok_or(InstallerError::InstallFailed)
}

fn allowed_sibling(path: &Path, scope: &AuthorizationScope) -> bool {
    if !safe_absolute(path) || path.parent() != Some(scope.install_root.as_path()) {
        return false;
    }
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name == "Beaver.app"
        || [
            ".Beaver.app.stage-",
            ".Beaver.app.backup-",
            ".Beaver.app.failed-",
        ]
        .iter()
        .any(|prefix| {
            name.strip_prefix(prefix).is_some_and(|suffix| {
                suffix.len() == 32
                    && suffix
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            })
        })
}

fn safe_absolute(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|component| component == Component::ParentDir)
        && !path.to_string_lossy().chars().any(char::is_control)
}

fn system_tool_is_regular(path: &Path) -> bool {
    fs_metadata(path)
        .is_some_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
}

fn fs_metadata(path: &Path) -> Option<std::fs::Metadata> {
    std::fs::symlink_metadata(path).ok()
}
