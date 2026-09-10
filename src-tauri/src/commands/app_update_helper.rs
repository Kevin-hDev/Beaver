use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const MAX_HELPER_SIZE: u64 = 64 * 1024 * 1024;
const MAX_COPY_ATTEMPTS: usize = 8;

#[cfg(test)]
pub(crate) fn copy_helper(
    source: &Path,
    resource_root: &Path,
    temp_root: &Path,
) -> Result<TemporaryHelper, String> {
    copy_helper_while(source, resource_root, temp_root, || false)
}

pub(crate) fn copy_helper_while(
    source: &Path,
    resource_root: &Path,
    temp_root: &Path,
    is_cancelled: impl Fn() -> bool,
) -> Result<TemporaryHelper, String> {
    if is_cancelled() {
        return Err(install_error());
    }
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
    let mut copied = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        if is_cancelled() {
            return Err(install_error());
        }
        let read = Read::by_ref(&mut input)
            .take((MAX_HELPER_SIZE + 1).saturating_sub(copied))
            .read(&mut buffer)
            .map_err(|_| install_error())?;
        if read == 0 {
            break;
        }
        output
            .write_all(&buffer[..read])
            .map_err(|_| install_error())?;
        copied = copied
            .checked_add(u64::try_from(read).map_err(|_| install_error())?)
            .ok_or_else(install_error)?;
    }
    if copied != source_metadata.len() || copied > MAX_HELPER_SIZE {
        return Err(install_error());
    }
    if is_cancelled() {
        return Err(install_error());
    }
    output.flush().map_err(|_| install_error())?;
    if is_cancelled() {
        return Err(install_error());
    }
    output.sync_all().map_err(|_| install_error())?;
    if is_cancelled() {
        return Err(install_error());
    }
    set_executable_permissions(temporary.path())?;
    if is_cancelled() {
        return Err(install_error());
    }
    Ok(temporary)
}

fn create_helper_file(temp_root: &Path) -> Result<(TemporaryHelper, File), String> {
    let canonical_root = std::fs::canonicalize(temp_root).map_err(|_| install_error())?;
    for _ in 0..MAX_COPY_ATTEMPTS {
        let mut random = [0_u8; 32];
        crate::services::secure_random::fill(&mut random);
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

pub(crate) fn current_install_directory() -> Result<PathBuf, String> {
    let executable = std::env::current_exe()
        .and_then(std::fs::canonicalize)
        .map_err(|_| install_error())?;
    current_install_directory_for(&executable)
}

pub(crate) fn current_install_directory_for(executable: &Path) -> Result<PathBuf, String> {
    let executable = std::fs::canonicalize(executable).map_err(|_| install_error())?;
    let metadata = std::fs::symlink_metadata(&executable).map_err(|_| install_error())?;
    if !metadata.is_file() {
        return Err(install_error());
    }
    let name = executable.file_name().and_then(|name| name.to_str());
    if name != Some(main_executable_name()) && name != Some(cli_executable_name()) {
        return Err(install_error());
    }
    let directory = executable.parent().ok_or_else(install_error)?.to_path_buf();
    let main = directory.join(main_executable_name());
    let main_metadata = std::fs::symlink_metadata(&main).map_err(|_| install_error())?;
    if !main_metadata.is_file() || main_metadata.file_type().is_symlink() {
        return Err(install_error());
    }
    let main = std::fs::canonicalize(main).map_err(|_| install_error())?;
    if main.parent() != Some(directory.as_path()) {
        return Err(install_error());
    }
    Ok(directory)
}

pub(crate) fn cli_update_paths_for(executable: &Path) -> Result<(PathBuf, PathBuf), String> {
    let executable = std::fs::canonicalize(executable).map_err(|_| install_error())?;
    let working_directory = current_install_directory_for(&executable)?;
    #[cfg(target_os = "macos")]
    let resource_root = working_directory
        .parent()
        .map(|contents| contents.join("Resources"))
        .ok_or_else(install_error)?
        .canonicalize()
        .map_err(|_| install_error())?;
    #[cfg(not(target_os = "macos"))]
    let resource_root = cli_resource_directory()?;
    Ok((resource_root, working_directory))
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn cli_resource_directory() -> Result<PathBuf, String> {
    let package = tauri::utils::PackageInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION")
            .parse()
            .map_err(|_| install_error())?,
        authors: env!("CARGO_PKG_AUTHORS"),
        description: env!("CARGO_PKG_DESCRIPTION"),
        crate_name: env!("CARGO_PKG_NAME"),
    };
    tauri::utils::platform::resource_dir(&package, &tauri::utils::Env::default())
        .map_err(|_| install_error())
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

pub(crate) fn helper_resource_name() -> &'static str {
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

fn cli_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "beaver.exe"
    } else {
        "beaver"
    }
}

fn exe_suffix() -> &'static str {
    if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    }
}

pub(crate) struct TemporaryHelper {
    path: PathBuf,
}

impl TemporaryHelper {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn persist(self) {
        std::mem::forget(self);
    }
}

#[cfg(test)]
pub(crate) fn test_temporary_helper() -> TemporaryHelper {
    let (helper, file) = create_helper_file(&std::env::temp_dir()).expect("temporary helper");
    drop(file);
    helper
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
