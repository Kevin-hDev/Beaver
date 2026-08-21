use std::path::{Path, PathBuf};

use super::runtime_error::RuntimeError;
use super::runtime_manifest::{RuntimeManifest, MANIFEST_NAME};

pub(super) const STAMP_NAME: &str = ".requirements.sha256";
const MAX_WHEELHOUSE_ENTRIES: usize = 512;

pub(super) struct Wheelhouse {
    pub(super) path: PathBuf,
    pub(super) manifest: RuntimeManifest,
}

pub(super) fn for_source(source: &Path) -> Result<Option<Wheelhouse>, RuntimeError> {
    let Some(parent) = source.parent() else {
        return Ok(None);
    };
    read_wheelhouse(&super::paths::wheelhouse_beside(parent)).map(Some)
}

pub(super) fn sync_from_archive_parent(archive: &Path) -> Result<(), String> {
    sync(archive).map_err(|error| error.public_code().to_string())
}

fn sync(archive: &Path) -> Result<(), RuntimeError> {
    sync_at(
        archive,
        &super::paths::wheels_dir(),
        &super::paths::staged_wheels_dir(),
        &super::paths::previous_wheels_dir(),
    )
}

pub(super) fn sync_at(
    archive: &Path,
    dest: &Path,
    tmp: &Path,
    previous: &Path,
) -> Result<(), RuntimeError> {
    let parent = archive
        .parent()
        .ok_or(RuntimeError::WheelhouseUnavailable)?;
    let wheelhouse = read_wheelhouse(&super::paths::wheelhouse_beside(parent))?;
    let paths = || super::generational_publication::Paths {
        current: dest,
        staged: tmp,
        previous,
    };
    super::generational_publication::recover(
        paths(),
        super::generational_publication::RecoveryPolicy::CommitImmediately,
        RuntimeError::WheelhouseUnavailable,
    )?;
    std::fs::create_dir_all(tmp).map_err(|_| RuntimeError::WheelhouseUnavailable)?;

    for entry in
        std::fs::read_dir(&wheelhouse.path).map_err(|_| RuntimeError::WheelhouseUnavailable)?
    {
        let entry = entry.map_err(|_| RuntimeError::WheelhouseUnavailable)?;
        let file_type = entry
            .file_type()
            .map_err(|_| RuntimeError::WheelhouseUnavailable)?;
        if file_type.is_file() && is_allowed_file(&entry.path()) {
            std::fs::copy(entry.path(), tmp.join(entry.file_name()))
                .map_err(|_| RuntimeError::WheelhouseUnavailable)?;
        }
    }

    super::generational_publication::publish(
        paths(),
        super::generational_publication::RecoveryPolicy::CommitImmediately,
        RuntimeError::WheelhouseUnavailable,
    )
}

fn read_wheelhouse(path: &Path) -> Result<Wheelhouse, RuntimeError> {
    let manifest = RuntimeManifest::read_from(path)?;
    let stamp = read_stamp(path)?;
    if !manifest.matches_stamp(&stamp) {
        return Err(RuntimeError::ManifestInvalid);
    }
    let mut wheels = 0;
    for (index, entry) in std::fs::read_dir(path)
        .map_err(|_| RuntimeError::WheelhouseUnavailable)?
        .enumerate()
    {
        if index >= MAX_WHEELHOUSE_ENTRIES {
            return Err(RuntimeError::WheelhouseUnavailable);
        }
        let entry = entry.map_err(|_| RuntimeError::WheelhouseUnavailable)?;
        let file_type = entry
            .file_type()
            .map_err(|_| RuntimeError::WheelhouseUnavailable)?;
        if !file_type.is_file() || !is_allowed_file(&entry.path()) {
            return Err(RuntimeError::WheelhouseUnavailable);
        }
        wheels += usize::from(is_wheel(&entry.path()));
    }
    if wheels == 0 {
        return Err(RuntimeError::WheelhouseUnavailable);
    }
    Ok(Wheelhouse {
        path: path.to_path_buf(),
        manifest,
    })
}

fn read_stamp(path: &Path) -> Result<String, RuntimeError> {
    let bytes = super::private_file::read_bounded(&path.join(STAMP_NAME), 64)
        .map_err(|_| RuntimeError::ManifestInvalid)?;
    if bytes.len() != 64 {
        return Err(RuntimeError::ManifestInvalid);
    }
    String::from_utf8(bytes).map_err(|_| RuntimeError::ManifestInvalid)
}

fn is_allowed_file(path: &Path) -> bool {
    is_wheel(path)
        || path.file_name().and_then(|name| name.to_str()) == Some(STAMP_NAME)
        || path.file_name().and_then(|name| name.to_str()) == Some(MANIFEST_NAME)
}

fn is_wheel(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("whl"))
}
