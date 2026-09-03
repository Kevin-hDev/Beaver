use rand::RngCore;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::types::{ExtensionRecord, ExtensionUiArtifact, MAX_USER_EXTENSIONS};

const DIRECTORY: &str = "extensions-ui";
const MAX_ROOT_ENTRIES: usize = MAX_USER_EXTENSIONS + 32;
const MAX_ARTIFACTS_PER_EXTENSION: usize = 32;

pub(super) struct StagingArtifact {
    path: PathBuf,
    output: PathBuf,
    temporary: PathBuf,
    committed: bool,
}

pub(super) fn prepare() -> Result<StagingArtifact, String> {
    let root = root();
    crate::services::private_store::ensure_private_dir(&root).map_err(|_| invalid())?;
    let mut random = [0_u8; 16];
    rand::rngs::OsRng
        .try_fill_bytes(&mut random)
        .map_err(|_| invalid())?;
    let path = root.join(format!(".staging-{}", hex::encode(random)));
    let output = path.join("output");
    let temporary = path.join("tmp");
    crate::services::private_store::ensure_private_dir(&output).map_err(|_| invalid())?;
    crate::services::private_store::ensure_private_dir(&temporary).map_err(|_| invalid())?;
    Ok(StagingArtifact {
        path,
        output,
        temporary,
        committed: false,
    })
}

pub(super) fn root() -> PathBuf {
    crate::services::paths::data_dir().join(DIRECTORY)
}

pub(super) fn artifact_path(extension_id: &str, manifest_sha: &str) -> Result<PathBuf, String> {
    super::validation::identifier(extension_id)?;
    if !valid_token(manifest_sha, 64) {
        return Err(invalid());
    }
    let root = root();
    validate_directory_if_present(&root, None)?;
    let parent = root.join(extension_id);
    validate_directory_if_present(&parent, Some(&root))?;
    Ok(parent.join(manifest_sha))
}

pub(super) fn remove(record: &ExtensionRecord) -> Result<(), String> {
    let Some(artifact) = &record.ui_artifact else {
        return Ok(());
    };
    let path = artifact_path(&record.manifest.id, &artifact.manifest_sha256)?;
    if path.exists() {
        std::fs::remove_dir_all(&path).map_err(|_| invalid())?;
    }
    remove_empty_parent(&path)
}

pub(super) fn unreferenced(records: &[ExtensionRecord]) -> Result<(), String> {
    let root = root();
    if !root.exists() {
        return Ok(());
    }
    let root = dunce::canonicalize(root).map_err(|_| invalid())?;
    let referenced = records
        .iter()
        .filter_map(|record| {
            let artifact = record.ui_artifact.as_ref()?;
            let path = artifact_path(&record.manifest.id, &artifact.manifest_sha256).ok()?;
            dunce::canonicalize(path).ok()
        })
        .collect::<HashSet<_>>();
    for (index, entry) in std::fs::read_dir(&root).map_err(|_| invalid())?.enumerate() {
        if index >= MAX_ROOT_ENTRIES {
            return Err(invalid());
        }
        let path = entry.map_err(|_| invalid())?.path();
        cleanup_entry(&path, &referenced)?;
    }
    Ok(())
}

pub(super) fn unreferenced_from_registry() -> Result<(), String> {
    unreferenced(&super::registry::list()?)
}

impl StagingArtifact {
    pub(super) fn output(&self) -> &Path {
        &self.output
    }
    pub(super) fn temporary(&self) -> &Path {
        &self.temporary
    }

    pub(super) fn commit(
        mut self,
        extension_id: &str,
        artifact: &ExtensionUiArtifact,
    ) -> Result<PathBuf, String> {
        let bytes = super::ui_artifact::manifest_bytes(artifact)?;
        crate::services::private_store::atomic_write(&self.output.join("manifest.json"), &bytes)
            .map_err(|_| invalid())?;
        super::ui_artifact::verify_at(&self.output, artifact)?;
        let destination = artifact_path(extension_id, &artifact.manifest_sha256)?;
        let parent = destination.parent().ok_or_else(invalid)?;
        crate::services::private_store::ensure_private_dir(parent).map_err(|_| invalid())?;
        if destination.exists() {
            if super::ui_artifact::verify_at(&destination, artifact).is_err() {
                std::fs::remove_dir_all(&destination).map_err(|_| invalid())?;
                std::fs::rename(&self.output, &destination).map_err(|_| invalid())?;
            }
        } else {
            std::fs::rename(&self.output, &destination).map_err(|_| invalid())?;
        }
        self.committed = true;
        let _ = std::fs::remove_dir_all(&self.path);
        Ok(destination)
    }
}

fn validate_directory_if_present(path: &Path, expected_root: Option<&Path>) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(invalid()),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(invalid());
    }
    if let Some(expected_root) = expected_root {
        let root = dunce::canonicalize(expected_root).map_err(|_| invalid())?;
        let candidate = dunce::canonicalize(path).map_err(|_| invalid())?;
        if !candidate.starts_with(root) {
            return Err(invalid());
        }
    }
    Ok(())
}

impl Drop for StagingArtifact {
    fn drop(&mut self) {
        if !self.committed {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

fn cleanup_entry(path: &Path, referenced: &HashSet<PathBuf>) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| invalid())?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if metadata.is_dir() && name.starts_with(".staging-") {
        return std::fs::remove_dir_all(path).map_err(|_| invalid());
    }
    if !metadata.is_dir() || super::validation::identifier(name).is_err() {
        return Ok(());
    }
    for (index, child) in std::fs::read_dir(path).map_err(|_| invalid())?.enumerate() {
        if index >= MAX_ARTIFACTS_PER_EXTENSION {
            return Err(invalid());
        }
        let child = child.map_err(|_| invalid())?.path();
        let child_name = child
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if valid_token(child_name, 64) && !referenced.contains(&child) {
            std::fs::remove_dir_all(&child).map_err(|_| invalid())?;
        }
    }
    remove_empty_parent(path)
}

fn remove_empty_parent(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent != root()
        && parent.exists()
        && std::fs::read_dir(parent)
            .map_err(|_| invalid())?
            .next()
            .is_none()
    {
        std::fs::remove_dir(parent).map_err(|_| invalid())?;
    }
    Ok(())
}

fn valid_token(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn invalid() -> String {
    super::ui_contract::UI_DIAGNOSTIC_UI_ARTIFACT_INVALID.to_string()
}
