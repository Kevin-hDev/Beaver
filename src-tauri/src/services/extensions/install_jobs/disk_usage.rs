//! Count only paths derived from the private ownership journal, with bounded traversal.
use super::{checkpoint::InstallCheckpoint, InstallInterruption};
use std::path::{Path, PathBuf};

const MAX_DEPTH: usize = 64;
const MAX_ENTRIES: usize = super::super::managed_tree::MAX_ENTRIES;

pub(super) fn measure(checkpoint: &InstallCheckpoint) -> Result<u64, InstallInterruption> {
    if !super::checkpoint::valid_token(&checkpoint.token) {
        return Err(InstallInterruption::Failed);
    }
    let token = &checkpoint.token;
    let mut roots = vec![
        super::super::managed_store::root().join(format!(".staging-{token}")),
        super::super::ui_artifact_store::root().join(format!(".staging-{token}")),
    ];
    if let Some(record) = &checkpoint.record {
        if super::super::installer::is_managed(record) {
            let root = super::super::managed_store::install_root(record)
                .map_err(|_| InstallInterruption::Failed)?;
            if root.file_name().and_then(|value| value.to_str()) != Some(token) {
                return Err(InstallInterruption::Failed);
            }
            roots.push(root);
        }
        if let Some(artifact) = &record.ui_artifact {
            roots.push(
                super::super::ui_artifact_store::artifact_path(
                    &record.manifest.id,
                    &artifact.manifest_sha256,
                )
                .map_err(|_| InstallInterruption::Failed)?,
            );
        }
    }
    measure_roots(&roots)
}

pub(super) fn measure_roots(roots: &[PathBuf]) -> Result<u64, InstallInterruption> {
    if roots.len() > 4 {
        return Err(InstallInterruption::Failed);
    }
    let mut entries = 0;
    let mut bytes = 0_u64;
    for root in roots {
        let Some(metadata) = read_metadata(root)? else {
            continue;
        };
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(InstallInterruption::Failed);
        }
        let mut pending = vec![(root.clone(), 0)];
        while let Some((directory, depth)) = pending.pop() {
            if depth > MAX_DEPTH {
                return Err(InstallInterruption::Failed);
            }
            let children = match std::fs::read_dir(&directory) {
                Ok(children) => children,
                // Producers may atomically move an owned directory during a sample.
                // The forced sample after they stop is the final authority.
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => return Err(InstallInterruption::Failed),
            };
            for child in children {
                entries += 1;
                if entries > MAX_ENTRIES {
                    return Err(InstallInterruption::Failed);
                }
                let path = child.map_err(|_| InstallInterruption::Failed)?.path();
                let Some(metadata) = read_metadata(&path)? else {
                    continue;
                };
                if metadata.file_type().is_symlink() {
                    return Err(InstallInterruption::Failed);
                }
                if metadata.is_dir() {
                    pending.push((path, depth + 1));
                } else if metadata.is_file() {
                    bytes = bytes
                        .checked_add(metadata.len())
                        .ok_or(InstallInterruption::Failed)?;
                } else {
                    return Err(InstallInterruption::Failed);
                }
            }
        }
    }
    Ok(bytes)
}

fn read_metadata(path: &Path) -> Result<Option<std::fs::Metadata>, InstallInterruption> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(InstallInterruption::Failed),
    }
}
