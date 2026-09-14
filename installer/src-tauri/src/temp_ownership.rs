use crate::error::InstallerError;
use serde::Deserialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use subtle::ConstantTimeEq;

pub const OWNER_MARKER: &str = ".beaver-installer-owner.json";
const RUN_PREFIX: &str = "beaver-install-";
const MAX_MARKER_BYTES: u64 = 128;
const MAX_TEMP_RUNS: usize = 128;
const MAX_RUN_ENTRIES: usize = 4_096;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerMarker {
    schema: u8,
    #[serde(rename = "runId")]
    run_id: String,
}

pub struct OwnedTempRun {
    path: PathBuf,
    cleanup_on_drop: AtomicBool,
}

impl OwnedTempRun {
    pub fn adopt(temp_root: &Path, work_dir: &Path, run_id: &str) -> Result<Self, InstallerError> {
        validate_run(temp_root, work_dir, run_id)?;
        Ok(Self {
            path: work_dir.to_path_buf(),
            cleanup_on_drop: AtomicBool::new(true),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn preserve_failure_trace(
        &self,
        trace: crate::trace::InstallerTrace,
    ) -> Result<PathBuf, InstallerError> {
        trace.preserve(self.run_id())
    }

    pub fn cleanup(&self) -> Result<(), InstallerError> {
        let root = self.path.parent().ok_or(InstallerError::CleanupFailed)?;
        validate_run(root, &self.path, self.run_id())?;
        validate_tree(&self.path)?;
        fs::remove_dir_all(&self.path).map_err(|_| InstallerError::CleanupFailed)
    }

    pub fn mark_active(&self) -> Result<(), InstallerError> {
        crate::temp_activity::mark(&self.path)
    }

    #[cfg(any(target_os = "windows", test))]
    pub(crate) fn disarm_cleanup(&self) {
        self.cleanup_on_drop.store(false, Ordering::Release);
    }

    fn run_id(&self) -> &str {
        run_id_from_name(&self.path).expect("validated owned run")
    }
}

impl Drop for OwnedTempRun {
    fn drop(&mut self) {
        if self.cleanup_on_drop.load(Ordering::Acquire) {
            let _ = self.cleanup();
        }
    }
}

pub fn purge_orphans(temp_root: &Path, current_run_id: &str) {
    let Ok(entries) = fs::read_dir(temp_root) else {
        return;
    };
    let mut candidates = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(run_id) = run_id_from_name(&path) else {
            continue;
        };
        if candidates == MAX_TEMP_RUNS {
            break;
        }
        candidates += 1;
        if constant_time_eq(run_id, current_run_id) {
            continue;
        }
        if validate_run(temp_root, &path, run_id).is_err() || validate_tree(&path).is_err() {
            eprintln!("installer-orphan-validation-failed");
            continue;
        }
        if crate::temp_activity::is_active(&path) {
            continue;
        }
        if fs::remove_dir_all(path).is_err() {
            eprintln!("installer-orphan-cleanup-failed");
        }
    }
}

fn validate_run(temp_root: &Path, work_dir: &Path, run_id: &str) -> Result<(), InstallerError> {
    let root = temp_root
        .canonicalize()
        .map_err(|_| InstallerError::CleanupFailed)?;
    let metadata = fs::symlink_metadata(work_dir).map_err(|_| InstallerError::CleanupFailed)?;
    if !metadata.is_dir() || is_link_or_reparse(&metadata) || !owned_by_current_user(&metadata) {
        return Err(InstallerError::CleanupFailed);
    }
    let canonical = work_dir
        .canonicalize()
        .map_err(|_| InstallerError::CleanupFailed)?;
    if canonical.parent() != Some(root.as_path())
        || work_dir.file_name().and_then(|name| name.to_str())
            != Some(format!("{RUN_PREFIX}{run_id}").as_str())
    {
        return Err(InstallerError::CleanupFailed);
    }
    validate_marker(&canonical.join(OWNER_MARKER), run_id)
}

fn validate_marker(path: &Path, run_id: &str) -> Result<(), InstallerError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| InstallerError::CleanupFailed)?;
    if !metadata.is_file()
        || metadata.len() > MAX_MARKER_BYTES
        || is_link_or_reparse(&metadata)
        || !owned_by_current_user(&metadata)
    {
        return Err(InstallerError::CleanupFailed);
    }
    let mut contents = String::new();
    crate::temp_activity::open_without_follow(path)
        .map_err(|_| InstallerError::CleanupFailed)?
        .take(MAX_MARKER_BYTES + 1)
        .read_to_string(&mut contents)
        .map_err(|_| InstallerError::CleanupFailed)?;
    let marker: OwnerMarker =
        serde_json::from_str(&contents).map_err(|_| InstallerError::CleanupFailed)?;
    if marker.schema != 1 || !constant_time_eq(&marker.run_id, run_id) {
        return Err(InstallerError::CleanupFailed);
    }
    Ok(())
}

fn validate_tree(root: &Path) -> Result<(), InstallerError> {
    let mut pending = vec![root.to_path_buf()];
    let mut visited = 0;
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).map_err(|_| InstallerError::CleanupFailed)? {
            visited += 1;
            if visited > MAX_RUN_ENTRIES {
                return Err(InstallerError::CleanupFailed);
            }
            let path = entry.map_err(|_| InstallerError::CleanupFailed)?.path();
            let metadata =
                fs::symlink_metadata(&path).map_err(|_| InstallerError::CleanupFailed)?;
            if is_link_or_reparse(&metadata) || !owned_by_current_user(&metadata) {
                return Err(InstallerError::CleanupFailed);
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if !metadata.is_file() {
                return Err(InstallerError::CleanupFailed);
            }
        }
    }
    Ok(())
}

fn run_id_from_name(path: &Path) -> Option<&str> {
    let name = path.file_name()?.to_str()?;
    let run_id = name.strip_prefix(RUN_PREFIX)?;
    (run_id.len() == 32
        && run_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
    .then_some(run_id)
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    // Un identifiant de run valide fait 32 octets ; les longueurs invalides restent comparées sans sortie précoce.
    let lengths = (left.len() as u64).ct_eq(&(right.len() as u64));
    let mut difference = 0_u8;
    for index in 0..32 {
        difference |= left.as_bytes().get(index).copied().unwrap_or_default()
            ^ right.as_bytes().get(index).copied().unwrap_or_default();
    }
    bool::from(lengths & difference.ct_eq(&0))
}

#[cfg(unix)]
fn owned_by_current_user(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    metadata.uid() == unsafe { libc::geteuid() }
}

#[cfg(windows)]
fn owned_by_current_user(_metadata: &fs::Metadata) -> bool {
    true
}

#[cfg(unix)]
fn is_link_or_reparse(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(windows)]
fn is_link_or_reparse(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
}
