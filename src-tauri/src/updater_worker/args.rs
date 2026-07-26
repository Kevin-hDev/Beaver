use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};

use super::{Platform, WorkerError};

const MAX_ARGUMENTS: usize = 6;
const MAX_ARGUMENT_LENGTH: usize = 4096;
const MAX_ASSET_SIZE: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ParsedArgs {
    pub(crate) asset: PathBuf,
    pub(crate) parent_pid: u32,
}

pub(crate) fn parse_from<I, S>(
    arguments: I,
    temp_root: &Path,
    platform: Platform,
) -> Result<ParsedArgs, WorkerError>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut values = Vec::with_capacity(5);
    for argument in arguments {
        if values.len() == MAX_ARGUMENTS {
            return Err(WorkerError);
        }
        let value = argument.into();
        if os_length(&value) > MAX_ARGUMENT_LENGTH {
            return Err(WorkerError);
        }
        values.push(value);
    }
    if values.len() != 5 || values[1] != "--apply-update" || values[3] != "--parent-pid" {
        return Err(WorkerError);
    }
    let parent_pid = parse_pid(&values[4])?;
    let asset = validate_asset(Path::new(&values[2]), temp_root, platform)?;
    Ok(ParsedArgs { asset, parent_pid })
}

fn parse_pid(raw: &OsStr) -> Result<u32, WorkerError> {
    let text = raw.to_str().ok_or(WorkerError)?;
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(WorkerError);
    }
    text.parse::<u32>()
        .ok()
        .filter(|pid| *pid > 0)
        .ok_or(WorkerError)
}

fn validate_asset(
    path: &Path,
    temp_root: &Path,
    platform: Platform,
) -> Result<PathBuf, WorkerError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| component == Component::ParentDir)
    {
        return Err(WorkerError);
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|_| WorkerError)?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > MAX_ASSET_SIZE
    {
        return Err(WorkerError);
    }
    validate_asset_name(path, platform)?;
    let canonical_root = std::fs::canonicalize(temp_root).map_err(|_| WorkerError)?;
    let canonical_asset = std::fs::canonicalize(path).map_err(|_| WorkerError)?;
    if !canonical_asset.starts_with(&canonical_root) {
        return Err(WorkerError);
    }
    Ok(canonical_asset)
}

fn validate_asset_name(path: &Path, platform: Platform) -> Result<(), WorkerError> {
    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or(WorkerError)?;
    let suffix = format!(".{}", platform.extension());
    let uuid = name
        .strip_prefix(super::UPDATE_TEMP_PREFIX)
        .and_then(|name| name.strip_prefix('-'))
        .and_then(|name| name.strip_suffix(&suffix))
        .ok_or(WorkerError)?;
    let value = uuid::Uuid::parse_str(uuid).map_err(|_| WorkerError)?;
    if value.get_version_num() != 4 {
        return Err(WorkerError);
    }
    Ok(())
}

fn os_length(value: &OsStr) -> usize {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        return value.as_bytes().len();
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        return value.encode_wide().take(MAX_ARGUMENT_LENGTH + 1).count();
    }
    #[allow(unreachable_code)]
    value
        .to_string_lossy()
        .chars()
        .take(MAX_ARGUMENT_LENGTH + 1)
        .count()
}

impl Platform {
    fn extension(self) -> &'static str {
        match self {
            Self::Macos => "dmg",
            Self::Windows => "exe",
            Self::Linux => "deb",
        }
    }
}

#[cfg(test)]
#[path = "args_tests.rs"]
mod tests;
