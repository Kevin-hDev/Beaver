//! Delete only deterministic paths owned by this job, after its producer stopped.
use super::checkpoint::InstallCheckpoint;
use std::path::Path;

pub(super) fn run(checkpoint: &InstallCheckpoint) -> Result<(), String> {
    if checkpoint.native_process.is_some() || checkpoint.cleanup_unconfirmed {
        return Err(super::limits::UNAVAILABLE.into());
    }
    let token = &checkpoint.token;
    if !super::checkpoint::valid_token(token) {
        return Err(super::limits::INVALID.into());
    }
    remove(&super::super::managed_store::root().join(format!(".staging-{token}")))?;
    remove(&super::super::ui_artifact_store::root().join(format!(".staging-{token}")))?;
    for (record, expected_token) in [
        (checkpoint.record.as_ref(), Some(token.as_str())),
        (checkpoint.previous.as_ref(), None),
    ] {
        let Some(record) = record else {
            continue;
        };
        let installed = super::super::registry::list()?
            .iter()
            .any(|current| current.source == record.source);
        if super::super::installer::is_managed(record) {
            let root = super::super::managed_store::install_root(record)?;
            if expected_token
                .is_some_and(|token| root.file_name().and_then(|name| name.to_str()) != Some(token))
            {
                return Err(super::limits::INVALID.into());
            }
        }
        if !installed && super::super::installer::is_managed(record) {
            super::super::managed_store::remove_record(record)?;
        }
        if let Some(artifact) = &record.ui_artifact {
            let referenced = super::super::registry::list()?.iter().any(|current| {
                current.manifest.id == record.manifest.id
                    && current
                        .ui_artifact
                        .as_ref()
                        .is_some_and(|value| value.manifest_sha256 == artifact.manifest_sha256)
            });
            if !referenced {
                super::super::ui_artifact_store::remove(record)?;
            }
        }
    }
    Ok(())
}

fn remove(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(super::limits::UNAVAILABLE.into()),
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            let root = path
                .parent()
                .ok_or(super::limits::INVALID)?
                .canonicalize()
                .map_err(|_| super::limits::INVALID)?;
            let actual = path.canonicalize().map_err(|_| super::limits::INVALID)?;
            if actual.parent() != Some(root.as_path()) {
                return Err(super::limits::INVALID.into());
            }
            std::fs::remove_dir_all(actual).map_err(|_| super::limits::UNAVAILABLE.into())
        }
        Ok(_) => Err(super::limits::INVALID.into()),
    }
}
