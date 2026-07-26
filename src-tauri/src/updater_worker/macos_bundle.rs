use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use super::verify::validate_regular_file;
use super::WorkerError;

const MAX_INFO_SIZE: u64 = 1024 * 1024;
const BUNDLE_IDENTIFIER: &str = "com.clgo.dash";
const EXECUTABLE_NAME: &str = "cl-go-dash";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BundleKind {
    Legacy,
    Beaver,
}

#[derive(Debug)]
pub(crate) struct ValidatedBundle {
    pub(crate) root: PathBuf,
    pub(crate) executable: PathBuf,
    pub(crate) kind: BundleKind,
}

pub(crate) fn validate_current(path: &Path) -> Result<ValidatedBundle, WorkerError> {
    let kind = match path.file_name().and_then(OsStr::to_str) {
        Some("CL-GO.app") => BundleKind::Legacy,
        Some("Beaver.app") => BundleKind::Beaver,
        _ => return Err(WorkerError),
    };
    validate_contents(path, kind)
}

pub(crate) fn validate_beaver_source(path: &Path) -> Result<ValidatedBundle, WorkerError> {
    if path.file_name() != Some(OsStr::new("Beaver.app")) {
        return Err(WorkerError);
    }
    validate_contents(path, BundleKind::Beaver)
}

pub(crate) fn validate_beaver_stage(path: &Path) -> Result<ValidatedBundle, WorkerError> {
    validate_contents(path, BundleKind::Beaver)
}

fn validate_contents(path: &Path, kind: BundleKind) -> Result<ValidatedBundle, WorkerError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| component == Component::ParentDir)
    {
        return Err(WorkerError);
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|_| WorkerError)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(WorkerError);
    }
    let root = std::fs::canonicalize(path).map_err(|_| WorkerError)?;
    validate_info(&root)?;
    let executable =
        validate_regular_file(&root.join("Contents/MacOS").join(EXECUTABLE_NAME), &root)?;
    Ok(ValidatedBundle {
        root,
        executable,
        kind,
    })
}

fn validate_info(root: &Path) -> Result<(), WorkerError> {
    let path = root.join("Contents/Info.plist");
    let metadata = std::fs::symlink_metadata(&path).map_err(|_| WorkerError)?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > MAX_INFO_SIZE
    {
        return Err(WorkerError);
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let file = options.open(path).map_err(|_| WorkerError)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_INFO_SIZE + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| WorkerError)?;
    if bytes.len() as u64 != metadata.len() {
        return Err(WorkerError);
    }
    let value = plist::Value::from_reader(std::io::Cursor::new(bytes)).map_err(|_| WorkerError)?;
    let dictionary = value.as_dictionary().ok_or(WorkerError)?;
    if dictionary
        .get("CFBundleIdentifier")
        .and_then(plist::Value::as_string)
        != Some(BUNDLE_IDENTIFIER)
        || dictionary
            .get("CFBundleExecutable")
            .and_then(plist::Value::as_string)
            != Some(EXECUTABLE_NAME)
    {
        return Err(WorkerError);
    }
    Ok(())
}

#[cfg(test)]
#[path = "macos_bundle_tests.rs"]
mod tests;
