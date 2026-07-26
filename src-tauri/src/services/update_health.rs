use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::time::SystemTime;

use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

pub const UPDATE_HEALTH_ARG: &str = "--clgo-update-health";
const MAX_ARGUMENTS: usize = 32;
const MAX_ARGUMENT_LENGTH: usize = 4096;
const MAX_ACK_FILES: usize = 8;
const MAX_DIRECTORY_ENTRIES: usize = 128;

pub fn acknowledge_from_args<I, S>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    acknowledge_in(args, &super::paths::data_dir())
}

fn acknowledge_in<I, S>(args: I, data_root: &Path) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let values = collect_bounded(args)?;
    let mut found = None;
    for (index, value) in values.iter().enumerate() {
        if value == UPDATE_HEALTH_ARG {
            if found.is_some() {
                return Err(health_error());
            }
            found = Some(index);
        }
    }
    let Some(index) = found else {
        return Ok(());
    };
    let raw = values
        .get(index + 1)
        .and_then(|value| value.to_str())
        .ok_or_else(health_error)?;
    let token = Zeroizing::new(raw.to_owned());
    if !valid_token(&token) {
        return Err(health_error());
    }
    let directory = data_root.join("update-health");
    validate_directory(data_root, &directory)?;
    prune_before_write(&directory, &token)?;
    super::private_store::atomic_write(&directory.join(format!("{}.ok", token.as_str())), b"ok")
        .map_err(|_| health_error())
}

fn collect_bounded<I, S>(args: I) -> Result<Vec<OsString>, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut values = Vec::with_capacity(8);
    for value in args {
        if values.len() == MAX_ARGUMENTS {
            return Err(health_error());
        }
        let value = value.as_ref();
        if os_length(value) > MAX_ARGUMENT_LENGTH {
            return Err(health_error());
        }
        values.push(value.to_owned());
    }
    Ok(values)
}

fn validate_directory(data_root: &Path, directory: &Path) -> Result<(), String> {
    let root_metadata = std::fs::symlink_metadata(data_root).map_err(|_| health_error())?;
    if !root_metadata.file_type().is_dir() || root_metadata.file_type().is_symlink() {
        return Err(health_error());
    }
    let root = std::fs::canonicalize(data_root).map_err(|_| health_error())?;
    let metadata = match std::fs::symlink_metadata(directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(health_error()),
    };
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(health_error());
    }
    let canonical = std::fs::canonicalize(directory).map_err(|_| health_error())?;
    if canonical.starts_with(root) {
        Ok(())
    } else {
        Err(health_error())
    }
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

fn prune_before_write(directory: &Path, current: &str) -> Result<(), String> {
    if !directory.exists() {
        return Ok(());
    }
    let mut candidates = Vec::with_capacity(MAX_ACK_FILES);
    for (index, entry) in std::fs::read_dir(directory)
        .map_err(|_| health_error())?
        .enumerate()
    {
        if index == MAX_DIRECTORY_ENTRIES {
            return Err(health_error());
        }
        let entry = entry.map_err(|_| health_error())?;
        if let Some(candidate) = ack_candidate(&entry, current)? {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(|left, right| {
        right
            .modified
            .cmp(&left.modified)
            .then_with(|| right.name.cmp(&left.name))
    });
    for candidate in candidates.into_iter().skip(MAX_ACK_FILES - 1) {
        std::fs::remove_file(candidate.path).map_err(|_| health_error())?;
    }
    Ok(())
}

struct AckCandidate {
    path: std::path::PathBuf,
    name: String,
    modified: SystemTime,
}

fn ack_candidate(entry: &std::fs::DirEntry, current: &str) -> Result<Option<AckCandidate>, String> {
    let name = entry.file_name();
    let Some(name) = name.to_str() else {
        return Ok(None);
    };
    let Some(token) = name.strip_suffix(".ok") else {
        return Ok(None);
    };
    if !valid_token(token) {
        return Ok(None);
    }
    if constant_time_eq(token, current) {
        return Err(health_error());
    }
    let metadata = std::fs::symlink_metadata(entry.path()).map_err(|_| health_error())?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(health_error());
    }
    Ok(Some(AckCandidate {
        path: entry.path(),
        name: name.to_owned(),
        modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
    }))
}

fn valid_token(token: &str) -> bool {
    token.len() == 64
        && token
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    let lengths = (left.len() as u64).ct_eq(&(right.len() as u64));
    let mut difference = 0_u8;
    for index in 0..64 {
        difference |= left.as_bytes().get(index).copied().unwrap_or_default()
            ^ right.as_bytes().get(index).copied().unwrap_or_default();
    }
    bool::from(lengths & difference.ct_eq(&0))
}

fn health_error() -> String {
    "initialisation de la mise à jour impossible".to_string()
}

#[cfg(test)]
#[path = "update_health_tests.rs"]
mod tests;
