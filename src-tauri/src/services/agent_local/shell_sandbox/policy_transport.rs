use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "policy.json";
const DIGEST_ENV: &str = "BEAVER_INTERNAL_SANDBOX_POLICY";
const MAX_BYTES: u64 = (super::super::directory_access::MAX_WORKSPACE_ROOTS
    * (super::super::directory_access::MAX_PATH_CHARS * 6 + 4)) as u64;

pub(super) fn write(
    command: &mut tokio::process::Command,
    temp_dir: &Path,
    roots: &[PathBuf],
) -> Result<(), String> {
    command.env(DIGEST_ENV, store(temp_dir, roots)?);
    Ok(())
}

#[cfg(unix)]
pub(super) fn write_std(
    command: &mut std::process::Command,
    temp_dir: &Path,
    roots: &[PathBuf],
) -> Result<(), String> {
    command.env(DIGEST_ENV, store(temp_dir, roots)?);
    Ok(())
}

fn store(temp_dir: &Path, roots: &[PathBuf]) -> Result<String, String> {
    let values = roots
        .iter()
        .map(|path| path.to_str().map(ToString::to_string))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(super::launch::sandbox_error)?;
    let bytes = serde_json::to_vec(&values).map_err(|_| super::launch::sandbox_error())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(super::launch::sandbox_error());
    }
    crate::services::private_store::atomic_write(&temp_dir.join(FILE_NAME), &bytes)?;
    Ok(digest(&bytes))
}

pub(super) fn take(temp_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let path = temp_dir.join(FILE_NAME);
    let metadata = path.symlink_metadata().map_err(|_| super::launch::sandbox_error())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_BYTES {
        return Err(super::launch::sandbox_error());
    }
    let bytes = std::fs::read(&path).map_err(|_| super::launch::sandbox_error())?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > MAX_BYTES {
        return Err(super::launch::sandbox_error());
    }
    let expected = std::env::var(DIGEST_ENV).map_err(|_| super::launch::sandbox_error())?;
    if !constant_time_eq(digest(&bytes).as_bytes(), expected.as_bytes()) {
        return Err(super::launch::sandbox_error());
    }
    std::fs::remove_file(path).map_err(|_| super::launch::sandbox_error())?;
    // SAFETY: le helper s’exécute avant Tauri et avant la création de threads.
    unsafe { std::env::remove_var(DIGEST_ENV) };
    let roots: Vec<String> =
        serde_json::from_slice(&bytes).map_err(|_| super::launch::sandbox_error())?;
    if roots.len() > super::super::directory_access::MAX_WORKSPACE_ROOTS {
        return Err(super::launch::sandbox_error());
    }
    super::super::directory_access::transported_roots_from_paths(roots)
        .map_err(|_| super::launch::sandbox_error())
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    let max = left.len().max(right.len());
    for index in 0..max {
        difference |= usize::from(left.get(index).copied().unwrap_or(0)
            ^ right.get(index).copied().unwrap_or(0));
    }
    difference == 0
}
