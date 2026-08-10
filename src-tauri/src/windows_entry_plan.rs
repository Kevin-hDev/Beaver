use std::path::{Path, PathBuf};

#[path = "windows_entry_arguments.rs"]
mod arguments;
#[cfg(not(feature = "windows-tests"))]
pub(crate) use arguments::{bootstrap_arguments, classify_bootstrap, BootstrapRole};

const APPLICATION_MODULE_FILE: &str = "cl_go_dash_lib.dll";
const DEVELOPMENT_BOOTSTRAP_FILE: &str = "cl_go_dash_lib.exe";
const MAX_APPLICATION_DLL_BYTES: u64 = 512 * 1024 * 1024;
pub(crate) const MAX_BOOTSTRAP_BYTES: u64 = 32 * 1024 * 1024;

pub(crate) fn stage_application_module(root: &Path) -> Result<PathBuf, ()> {
    let dependency_root = root.join("deps").canonicalize().map_err(|_| ())?;
    if !dependency_root.starts_with(root) {
        return Err(());
    }
    let source = checked_file(
        &dependency_root,
        APPLICATION_MODULE_FILE,
        MAX_APPLICATION_DLL_BYTES,
    )?;
    let destination = root.join(APPLICATION_MODULE_FILE);
    replace_file(
        &source,
        &destination,
        &root.join("cl_go_dash_lib.dll.tmp"),
        root,
    )?;
    checked_file(root, APPLICATION_MODULE_FILE, MAX_APPLICATION_DLL_BYTES)
}

pub(crate) fn stage_bootstrap_executable(root: &Path, source: &Path) -> Result<PathBuf, ()> {
    if checked_file(root, "bootstrap.exe", MAX_BOOTSTRAP_BYTES)? != source {
        return Err(());
    }
    let destination = root.join(DEVELOPMENT_BOOTSTRAP_FILE);
    replace_file(
        source,
        &destination,
        &root.join("cl_go_dash_lib.exe.tmp"),
        root,
    )?;
    checked_file(root, DEVELOPMENT_BOOTSTRAP_FILE, MAX_BOOTSTRAP_BYTES)
}

pub(crate) fn checked_file(root: &Path, name: &str, max_bytes: u64) -> Result<PathBuf, ()> {
    let path = root.join(name).canonicalize().map_err(|_| ())?;
    let metadata = path.symlink_metadata().map_err(|_| ())?;
    if !path.starts_with(root)
        || !metadata.file_type().is_file()
        || metadata.len() == 0
        || metadata.len() > max_bytes
    {
        return Err(());
    }
    Ok(path)
}

fn replace_file(
    source: &Path,
    destination: &Path,
    temporary: &Path,
    root: &Path,
) -> Result<(), ()> {
    if destination.parent() != Some(root) || temporary.parent() != Some(root) {
        return Err(());
    }
    let _ = std::fs::remove_file(temporary);
    std::fs::copy(source, temporary).map_err(|_| ())?;
    if destination.exists() {
        std::fs::remove_file(destination).map_err(|_| ())?;
    }
    std::fs::rename(temporary, destination).map_err(|_| ())
}

#[cfg(test)]
#[path = "windows_entry_tests.rs"]
mod tests;
