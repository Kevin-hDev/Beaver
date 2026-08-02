use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use tokio::process::Command;

const HELPER_ARG: &str = "--beaver-shell-sandbox";
const PROFILE_CAPTURE_ARG: &str = "--profile-capture";
const TEMP_DIRECTORY: &str = "shell-sandboxes";

pub struct PreparedShellCommand {
    pub command: Command,
    pub cleanup_dir: Option<PathBuf>,
}

#[cfg(unix)]
pub(crate) struct PreparedProfileCapture {
    command: std::process::Command,
    cleanup_dir: PathBuf,
}

#[cfg(unix)]
impl PreparedProfileCapture {
    pub fn command_mut(&mut self) -> &mut std::process::Command {
        &mut self.command
    }
}

#[cfg(unix)]
impl Drop for PreparedProfileCapture {
    fn drop(&mut self) {
        cleanup_one(&self.cleanup_dir);
    }
}

pub fn prepare_command(
    shell: &OsStr,
    arguments: &[String],
    working_dir: &Path,
) -> Result<PreparedShellCommand, String> {
    let roots = super::super::directory_access::configured_roots()?;
    super::super::directory_access::ensure_allowed_in_roots(working_dir, &roots)?;
    let shell_path = super::super::shell_environment::value();
    if roots.iter().any(|root| root.parent().is_none()) {
        let mut command = Command::new(shell);
        command.args(arguments).env("PATH", &shell_path);
        return Ok(PreparedShellCommand {
            command,
            cleanup_dir: None,
        });
    }

    let temp_dir = create_sandbox_temp()?;
    let executable = match helper_executable() {
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
        .env("TEMP", &temp_dir)
        .env("TMPPREFIX", temp_dir.join("zsh"))
        .env("PATH", shell_path);
    Ok(PreparedShellCommand {
        command,
        cleanup_dir: Some(temp_dir),
    })
}

#[cfg(unix)]
pub(crate) fn prepare_profile_capture(
    shell: &Path,
    arguments: &[OsString],
    base_path: &OsStr,
) -> Result<PreparedProfileCapture, String> {
    let temp_dir = create_sandbox_temp()?;
    let executable = match helper_executable() {
        Ok(executable) => executable,
        Err(error) => {
            cleanup_one(&temp_dir);
            return Err(error);
        }
    };
    let mut command = std::process::Command::new(executable);
    command
        .arg(HELPER_ARG)
        .arg(PROFILE_CAPTURE_ARG)
        .arg(&temp_dir)
        .arg("--")
        .arg(shell)
        .args(arguments)
        .current_dir(&temp_dir)
        .env("TMPDIR", &temp_dir)
        .env("TMP", &temp_dir)
        .env("TEMP", &temp_dir)
        .env("TMPPREFIX", temp_dir.join("zsh"))
        .env("PATH", base_path);
    Ok(PreparedProfileCapture {
        command,
        cleanup_dir: temp_dir,
    })
}

fn helper_executable() -> Result<PathBuf, String> {
    let executable = std::env::current_exe()
        .map_err(|_| sandbox_error())
        .and_then(|path| dunce::canonicalize(path).map_err(|_| sandbox_error()))?;
    executable.is_file().then_some(executable).ok_or_else(sandbox_error)
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

pub(super) fn profile_capture_arg() -> &'static str {
    PROFILE_CAPTURE_ARG
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
    let mut entries = entries.flatten();
    for _ in 0..256 {
        let Some(entry) = entries.next() else { return };
        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_dir() && !kind.is_symlink()) {
            cleanup_one(&path);
        }
        #[cfg(windows)]
        if entry.file_type().is_ok_and(|kind| kind.is_file() && !kind.is_symlink()) {
            super::windows::cleanup_record_file(&path);
        }
    }
    if entries.next().is_some() {
        eprintln!("[shell-sandbox] stale cleanup incomplete");
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
