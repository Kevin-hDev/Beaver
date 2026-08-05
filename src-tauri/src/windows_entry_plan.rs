use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const APPLICATION_MODULE_FILE: &str = "cl_go_dash_lib.dll";
const DEVELOPMENT_BOOTSTRAP_FILE: &str = "cl_go_dash_lib.exe";
const MAX_APPLICATION_DLL_BYTES: u64 = 512 * 1024 * 1024;
pub(crate) const MAX_BOOTSTRAP_BYTES: u64 = 32 * 1024 * 1024;
const MAX_FORWARD_ARGS: usize = 64;
const MAX_ARG_UTF16: usize = 2_048;

pub(crate) fn bootstrap_arguments(
    forwarded: impl IntoIterator<Item = OsString>,
) -> Result<Vec<OsString>, ()> {
    let mut result = Vec::with_capacity(MAX_FORWARD_ARGS);
    for argument in forwarded.into_iter().take(MAX_FORWARD_ARGS + 1) {
        if result.len() == MAX_FORWARD_ARGS
            || argument.encode_wide().count() > MAX_ARG_UTF16
            || is_module_switch(&argument)
        {
            return Err(());
        }
        result.push(argument);
    }
    Ok(result)
}

fn is_module_switch(argument: &OsStr) -> bool {
    let Some(value) = argument.to_str() else {
        return true;
    };
    value.eq_ignore_ascii_case("--module")
        || value
            .get(..9)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("--module="))
}

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
