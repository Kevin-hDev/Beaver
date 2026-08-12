use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemExecutableError {
    InvalidSystemRoot,
    MissingExecutable,
}

#[cfg(windows)]
pub fn powershell() -> Result<PathBuf, SystemExecutableError> {
    let root = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .ok_or(SystemExecutableError::InvalidSystemRoot)?;
    powershell_from_root(&root)
}

#[cfg(windows)]
fn powershell_from_root(root: &Path) -> Result<PathBuf, SystemExecutableError> {
    if !root.is_absolute() {
        return Err(SystemExecutableError::InvalidSystemRoot);
    }
    let executable = root
        .join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");
    if !executable.is_absolute() || !executable.is_file() {
        return Err(SystemExecutableError::MissingExecutable);
    }
    Ok(executable)
}

#[cfg(all(test, windows))]
mod tests {
    #[test]
    fn powershell_resolution_uses_the_validated_system_root_not_path() {
        let root = tempfile::tempdir().unwrap();
        let poisoned_path = tempfile::tempdir().unwrap();
        let poisoned = poisoned_path.path().join("powershell.exe");
        std::fs::write(&poisoned, b"untrusted").unwrap();
        let executable = root
            .path()
            .join("System32")
            .join("WindowsPowerShell")
            .join("v1.0")
            .join("powershell.exe");
        std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
        std::fs::write(&executable, b"fixture").unwrap();

        let resolved = super::powershell_from_root(root.path()).expect("absolute PowerShell");

        assert!(resolved.is_absolute());
        assert_eq!(resolved, executable);
        assert_ne!(resolved, poisoned);
    }
}
