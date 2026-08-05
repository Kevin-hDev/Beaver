use rand::RngCore;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tauri::Manager;

const MAX_HELPER_SIZE: u64 = 64 * 1024 * 1024;
const MAX_COPY_ATTEMPTS: usize = 8;

pub(crate) fn spawn_update_helper(app: &tauri::AppHandle, asset: &Path) -> Result<(), String> {
    let resource_root = app.path().resource_dir().map_err(|_| install_error())?;
    let source = resource_root
        .join("target/updater-helper")
        .join(helper_resource_name());
    let helper = copy_helper(&source, &resource_root, &std::env::temp_dir())?;
    let working_directory = current_install_directory()?;
    let mut command = crate::services::background_command::new(helper.path());
    command
        .arg("--apply-update")
        .arg(asset)
        .arg("--parent-pid")
        .arg(std::process::id().to_string())
        .current_dir(working_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command.spawn().map_err(|_| install_error())?;
    helper.persist();
    Ok(())
}

fn copy_helper(
    source: &Path,
    resource_root: &Path,
    temp_root: &Path,
) -> Result<TemporaryHelper, String> {
    let canonical_root = std::fs::canonicalize(resource_root).map_err(|_| install_error())?;
    let source_metadata = std::fs::symlink_metadata(source).map_err(|_| install_error())?;
    if !source_metadata.file_type().is_file() || source_metadata.file_type().is_symlink() {
        return Err(install_error());
    }
    let canonical_source = std::fs::canonicalize(source).map_err(|_| install_error())?;
    if !canonical_source.starts_with(&canonical_root)
        || source_metadata.len() == 0
        || source_metadata.len() > MAX_HELPER_SIZE
    {
        return Err(install_error());
    }
    let mut input = File::open(&canonical_source).map_err(|_| install_error())?;
    let (temporary, mut output) = create_helper_file(temp_root)?;
    let copied = std::io::copy(
        &mut Read::by_ref(&mut input).take(MAX_HELPER_SIZE + 1),
        &mut output,
    )
    .map_err(|_| install_error())?;
    if copied != source_metadata.len() || copied > MAX_HELPER_SIZE {
        return Err(install_error());
    }
    output.flush().map_err(|_| install_error())?;
    output.sync_all().map_err(|_| install_error())?;
    set_executable_permissions(temporary.path())?;
    Ok(temporary)
}

fn create_helper_file(temp_root: &Path) -> Result<(TemporaryHelper, File), String> {
    let canonical_root = std::fs::canonicalize(temp_root).map_err(|_| install_error())?;
    for _ in 0..MAX_COPY_ATTEMPTS {
        let mut random = [0_u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut random);
        let name = format!("cl-go-dash-updater-{}{}", hex::encode(random), exe_suffix());
        let path = canonical_root.join(name);
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(file) => return Ok((TemporaryHelper { path }, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(install_error()),
        }
    }
    Err(install_error())
}

fn current_install_directory() -> Result<PathBuf, String> {
    let executable = std::env::current_exe().map_err(|_| install_error())?;
    if executable.file_name().and_then(|name| name.to_str()) != Some(main_executable_name()) {
        return Err(install_error());
    }
    let parent = executable.parent().ok_or_else(install_error)?;
    std::fs::canonicalize(parent).map_err(|_| install_error())
}

#[cfg(unix)]
fn set_executable_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        .map_err(|_| install_error())
}

#[cfg(not(unix))]
fn set_executable_permissions(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn helper_resource_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "cl-go-dash-updater.exe"
    } else {
        "cl-go-dash-updater"
    }
}

fn main_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "cl-go-dash.exe"
    } else {
        "cl-go-dash"
    }
}

fn exe_suffix() -> &'static str {
    if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    }
}

struct TemporaryHelper {
    path: PathBuf,
}

impl TemporaryHelper {
    fn path(&self) -> &Path {
        &self.path
    }

    fn persist(self) {
        std::mem::forget(self);
    }
}

impl Drop for TemporaryHelper {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn install_error() -> String {
    "update-install-error".to_string()
}

#[cfg(test)]
#[path = "app_update_helper_tests.rs"]
mod tests;
