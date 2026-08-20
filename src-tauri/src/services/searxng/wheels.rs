use std::path::{Path, PathBuf};

use super::runtime_error::RuntimeError;
use super::runtime_manifest::{RuntimeManifest, MANIFEST_NAME};

const STAMP_NAME: &str = ".requirements.sha256";

pub(super) struct Wheelhouse {
    pub(super) path: PathBuf,
    pub(super) manifest: RuntimeManifest,
}

pub(super) fn for_source(source: &Path) -> Result<Option<Wheelhouse>, RuntimeError> {
    let Some(parent) = source.parent() else {
        return Ok(None);
    };
    read_wheelhouse(&parent.join("wheels")).map(Some)
}

pub(super) fn sync_from_archive_parent(archive: &Path) -> Result<(), String> {
    sync(archive).map_err(|error| error.public_message().to_string())
}

fn sync(archive: &Path) -> Result<(), RuntimeError> {
    let parent = archive
        .parent()
        .ok_or(RuntimeError::WheelhouseUnavailable)?;
    let wheelhouse = read_wheelhouse(&parent.join("wheels"))?;
    let dest = super::paths::sidecar_dir().join("wheels");
    let tmp = super::paths::sidecar_dir().join("wheels.tmp");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).map_err(|_| RuntimeError::WheelhouseUnavailable)?;

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

    let _ = std::fs::remove_dir_all(&dest);
    std::fs::rename(&tmp, &dest).map_err(|_| RuntimeError::WheelhouseUnavailable)
}

fn read_wheelhouse(path: &Path) -> Result<Wheelhouse, RuntimeError> {
    let manifest = RuntimeManifest::read_from(path)?;
    let stamp = read_stamp(path)?;
    if !manifest.matches_stamp(&stamp) {
        return Err(RuntimeError::ManifestInvalid);
    }
    let mut wheels = 0;
    for entry in std::fs::read_dir(path).map_err(|_| RuntimeError::WheelhouseUnavailable)? {
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
    let metadata = std::fs::symlink_metadata(path.join(STAMP_NAME))
        .map_err(|_| RuntimeError::ManifestInvalid)?;
    if !metadata.file_type().is_file() || metadata.len() != 64 {
        return Err(RuntimeError::ManifestInvalid);
    }
    let stamp = std::fs::read_to_string(path.join(STAMP_NAME))
        .map_err(|_| RuntimeError::ManifestInvalid)?;
    Ok(stamp)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheelhouse_exposes_the_validated_manifest() {
        let (_parent, source, _wheels) = valid_wheelhouse();

        let result = for_source(&source);

        let Some(wheelhouse) = result.expect("wheelhouse") else {
            panic!("wheelhouse must be present");
        };
        assert_eq!(wheelhouse.manifest.python_major, 3);
        assert_eq!(wheelhouse.manifest.python_minor, 14);
    }

    #[test]
    fn wheelhouse_rejects_a_stamp_not_bound_to_the_manifest() {
        let (_parent, source, wheels) = valid_wheelhouse();
        std::fs::write(wheels.join(STAMP_NAME), "b".repeat(64)).unwrap();

        assert!(matches!(
            for_source(&source),
            Err(RuntimeError::ManifestInvalid)
        ));
    }

    #[test]
    fn wheelhouse_rejects_an_oversized_manifest_and_a_foreign_file() {
        let (_parent, source, wheels) = valid_wheelhouse();
        std::fs::write(wheels.join(MANIFEST_NAME), vec![b'x'; 513]).unwrap();
        assert!(matches!(
            for_source(&source),
            Err(RuntimeError::ManifestInvalid)
        ));

        std::fs::write(wheels.join(MANIFEST_NAME), manifest()).unwrap();
        std::fs::write(wheels.join("foreign.txt"), b"unexpected").unwrap();
        assert!(matches!(
            for_source(&source),
            Err(RuntimeError::WheelhouseUnavailable)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn wheelhouse_rejects_a_symlinked_manifest() {
        let (_parent, source, wheels) = valid_wheelhouse();
        let target = wheels.join("manifest-target");
        std::fs::write(&target, manifest()).unwrap();
        std::fs::remove_file(wheels.join(MANIFEST_NAME)).unwrap();
        std::os::unix::fs::symlink(target, wheels.join(MANIFEST_NAME)).unwrap();

        assert!(matches!(
            for_source(&source),
            Err(RuntimeError::ManifestInvalid)
        ));
    }

    fn valid_wheelhouse() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let parent = tempfile::tempdir().unwrap();
        let source = parent.path().join("source");
        std::fs::create_dir(&source).unwrap();
        let wheels = parent.path().join("wheels");
        std::fs::create_dir(&wheels).unwrap();
        std::fs::write(wheels.join("a.whl"), b"wheel").unwrap();
        std::fs::write(wheels.join(STAMP_NAME), "a".repeat(64)).unwrap();
        std::fs::write(wheels.join(MANIFEST_NAME), manifest()).unwrap();

        (parent, source, wheels)
    }

    fn manifest() -> &'static [u8] {
        br#"{"schema_version":1,"implementation":"cpython","major":3,"minor":14,"requirements_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#
    }
}
