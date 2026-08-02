use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use tokio::process::Command;

const HELPER_ARG: &str = "--beaver-shell-sandbox";
const TEMP_DIRECTORY: &str = "shell-sandboxes";

pub struct PreparedShellCommand {
    pub command: Command,
    pub cleanup_dir: Option<PathBuf>,
}

pub fn prepare_command(
    shell: &OsStr,
    arguments: &[String],
    working_dir: &Path,
) -> Result<PreparedShellCommand, String> {
    let roots = super::super::directory_access::configured_roots()?;
    super::super::directory_access::ensure_allowed_in_roots(working_dir, &roots)?;
    if roots.iter().any(|root| root.parent().is_none()) {
        let mut command = Command::new(shell);
        command.args(arguments);
        return Ok(PreparedShellCommand {
            command,
            cleanup_dir: None,
        });
    }

    let temp_dir = create_sandbox_temp()?;
    let executable = match std::env::current_exe()
        .map_err(|_| sandbox_error())
        .and_then(|path| dunce::canonicalize(path).map_err(|_| sandbox_error()))
    {
        Ok(executable) => executable,
        Err(error) => {
            cleanup_one(&temp_dir);
            return Err(error);
        }
    };
    if !executable.is_file() {
        cleanup_one(&temp_dir);
        return Err(sandbox_error());
    }
    let mut command = Command::new(executable);
    command
        .arg(HELPER_ARG)
        .arg(&temp_dir)
        .arg("--")
        .arg(shell)
        .args(arguments)
        .env("TMPDIR", &temp_dir)
        .env("TMP", &temp_dir)
        .env("TEMP", &temp_dir);
    Ok(PreparedShellCommand {
        command,
        cleanup_dir: Some(temp_dir),
    })
}

fn create_sandbox_temp() -> Result<PathBuf, String> {
    let root = sandbox_temp_root();
    super::super::super::private_store::ensure_private_dir(&root)?;
    let path = root.join(uuid::Uuid::new_v4().simple().to_string());
    super::super::super::private_store::ensure_private_dir(&path)?;
    dunce::canonicalize(path).map_err(|_| sandbox_error())
}

pub(super) fn sandbox_temp_root() -> PathBuf {
    super::super::super::paths::data_dir().join(TEMP_DIRECTORY)
}

pub(super) fn helper_arg() -> &'static str {
    HELPER_ARG
}

pub(super) fn sandbox_error() -> String {
    "Isolation du shell indisponible.".to_string()
}

pub async fn cleanup_temp(path: Option<PathBuf>) {
    let Some(path) = path else { return };
    let _ = tokio::task::spawn_blocking(move || cleanup_one(&path)).await;
}

pub fn cleanup_stale() {
    let root = sandbox_temp_root();
    let Ok(entries) = std::fs::read_dir(&root) else { return };
    for entry in entries.flatten().take(256) {
        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_dir() && !kind.is_symlink()) {
            cleanup_one(&path);
        }
        #[cfg(windows)]
        if entry.file_type().is_ok_and(|kind| kind.is_file() && !kind.is_symlink()) {
            super::windows::cleanup_record_file(&path);
        }
    }
}

fn cleanup_one(path: &Path) {
    #[cfg(windows)]
    super::windows::cleanup(path);
    let _ = std::fs::remove_dir_all(path);
}

pub(super) fn os_text(value: &OsStr) -> Result<OsString, String> {
    let text = value.to_str().ok_or_else(sandbox_error)?;
    if text.is_empty() || text.contains('\0') || text.chars().count() > 524_288 {
        return Err(sandbox_error());
    }
    Ok(value.to_os_string())
}
