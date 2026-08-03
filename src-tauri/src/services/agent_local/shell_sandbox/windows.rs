#[path = "windows/windows_acl.rs"]
mod windows_acl;
#[path = "windows/windows_process.rs"]
mod windows_process;
#[path = "windows/windows_profile.rs"]
mod windows_profile;

use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

const RECORD_SUFFIX: &str = "cleanup.json";
const MAX_ROOTS: usize = super::super::directory_access::MAX_ALLOWED_PATHS
    + 1
    + super::tool_roots::MAX_READ_ROOTS
    + super::tool_roots::MAX_WRITE_ROOTS;
const MAX_RECORD_BYTES: u64 =
    (MAX_ROOTS * (super::super::directory_access::MAX_PATH_CHARS * 6 + 4) + 512) as u64;

#[derive(Serialize, Deserialize)]
struct CleanupRecord {
    profile_name: String,
    roots: Vec<PathBuf>,
}

pub(super) fn run(
    executable: &Path,
    arguments: &[OsString],
    scope: &super::scope::Scope,
    temp_dir: &Path,
) -> Result<i32, String> {
    if scope.mode != super::scope::Mode::Workspace {
        return Err(error());
    }
    let writable_roots = &scope.roots;
    if writable_roots.len() > super::super::directory_access::MAX_ALLOWED_PATHS {
        return Err(error());
    }
    let tools = super::tool_roots::collect(writable_roots, &[], &[], Some(executable));
    let writable = writable_roots
        .iter()
        .cloned()
        .chain(std::iter::once(temp_dir.to_path_buf()))
        .chain(tools.write_dirs)
        .chain(tools.write_files)
        .collect::<Vec<_>>();
    let readable = tools
        .read_dirs
        .into_iter()
        .chain(tools.read_files)
        .collect::<Vec<_>>();
    let profile = windows_profile::Profile::create()?;
    let candidates = writable
        .iter()
        .chain(&readable)
        .cloned()
        .collect::<Vec<_>>();
    write_record(temp_dir, profile.name(), &candidates)?;
    for root in &writable {
        windows_acl::grant(root, profile.sid(), true)?;
    }
    let mut recorded = writable.clone();
    let private_store = dunce::canonicalize(crate::services::paths::data_dir()).ok();
    for root in &readable {
        if private_store.as_ref() == Some(root) {
            windows_acl::grant(root, profile.sid(), false)?;
            recorded.push(root.clone());
            continue;
        }
        // Les dossiers système sont déjà lisibles par les AppContainers et leur
        // DACL n'est généralement pas modifiable par un utilisateur standard.
        if windows_acl::grant(root, profile.sid(), false).is_ok() {
            recorded.push(root.clone());
        }
    }
    // En cas d'échec, le journal initial, plus large, reste sûr à rejouer.
    let _ = write_record(temp_dir, profile.name(), &recorded);
    windows_process::run(executable, arguments, profile.sid())
}

pub(super) fn cleanup(temp_dir: &Path) {
    let path = record_path(temp_dir);
    cleanup_record_file(&path);
}

pub(super) fn cleanup_record_file(path: &Path) {
    let Some(record) = read_record(path) else {
        let _ = std::fs::remove_file(path);
        return;
    };
    if let Ok(profile) = windows_profile::Profile::derive(&record.profile_name) {
        for root in &record.roots {
            let _ = windows_acl::revoke(root, profile.sid());
        }
    }
    windows_profile::delete(&record.profile_name);
    let _ = std::fs::remove_file(path);
}

fn write_record(temp_dir: &Path, profile_name: &str, roots: &[PathBuf]) -> Result<(), String> {
    let record = CleanupRecord {
        profile_name: profile_name.to_string(),
        roots: roots.to_vec(),
    };
    let bytes = serde_json::to_vec(&record).map_err(|_| error())?;
    if bytes.len() as u64 > MAX_RECORD_BYTES {
        return Err(error());
    }
    super::super::super::private_store::atomic_write(&record_path(temp_dir), &bytes)
}

fn read_record(path: &Path) -> Option<CleanupRecord> {
    let metadata = path.symlink_metadata().ok()?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_RECORD_BYTES {
        return None;
    }
    let record: CleanupRecord = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    if !windows_profile::valid_name(&record.profile_name)
        || record.roots.is_empty()
        || record.roots.len() > MAX_ROOTS
        || record.roots.iter().any(|root| !valid_root(root))
    {
        return None;
    }
    Some(record)
}

fn record_path(temp_dir: &Path) -> PathBuf {
    temp_dir.with_extension(RECORD_SUFFIX)
}

fn valid_root(path: &Path) -> bool {
    path.is_absolute()
        && path.as_os_str().to_string_lossy().chars().count()
            <= super::super::directory_access::MAX_PATH_CHARS
        && !path.components().any(|part| matches!(part, Component::ParentDir))
}

fn error() -> String {
    super::launch::sandbox_error()
}
