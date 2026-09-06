//! The lock and manifest are the replay authority; npm cache is only an optimization.
use super::{InstallControl, InstallInterruption, InstallRequest};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const MAX_LOCK_BYTES: u64 = 4 * 1024 * 1024;
const MANIFEST: &str = "package.json";
const LOCKS: [&str; 2] = ["npm-shrinkwrap.json", "package-lock.json"];

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::services::extensions) struct DependencyLock {
    relative_root: PathBuf,
    digest: String,
}

impl InstallControl {
    pub(in crate::services::extensions) fn resolved_git(
        &self,
        source: &super::super::source_validation::GitSource,
    ) -> Result<(), InstallInterruption> {
        let mut checkpoint = self.saved()?.ok_or(InstallInterruption::Failed)?;
        checkpoint.resolved_source = Some(InstallRequest::Git {
            locator: source.locator.clone(),
        });
        self.save_resolution(checkpoint)
    }
    pub(in crate::services::extensions) fn lock_dependencies(
        &self,
        root: &Path,
    ) -> Result<(), InstallInterruption> {
        let mut checkpoint = self.saved()?.ok_or(InstallInterruption::Failed)?;
        let staging = super::super::managed_store::root()
            .join(format!(".staging-{}", checkpoint.token))
            .canonicalize()
            .map_err(|_| InstallInterruption::Failed)?;
        let root = root
            .canonicalize()
            .map_err(|_| InstallInterruption::Failed)?;
        let relative_root = root
            .strip_prefix(&staging)
            .map_err(|_| InstallInterruption::Failed)?
            .to_owned();
        checkpoint.dependency_lock = Some(DependencyLock {
            relative_root,
            digest: digest(&root)?,
        });
        if let Some(InstallRequest::Npm { locator }) = &checkpoint.resolved_source {
            let source = super::super::source_validation::npm(locator)
                .map_err(|_| InstallInterruption::Failed)?;
            let manifest: serde_json::Value = serde_json::from_slice(&read(&root.join(MANIFEST))?)
                .map_err(|_| InstallInterruption::Failed)?;
            let version = manifest
                .get("dependencies")
                .and_then(|deps| deps.get(&source.package_name))
                .and_then(serde_json::Value::as_str)
                .ok_or(InstallInterruption::Failed)?;
            if !version.starts_with(|character: char| character.is_ascii_digit()) {
                return Err(InstallInterruption::Failed);
            }
            let pinned = format!("{}@{version}", source.package_name);
            super::super::source_validation::npm(&pinned)
                .map_err(|_| InstallInterruption::Failed)?;
            checkpoint.resolved_source = Some(InstallRequest::Npm { locator: pinned });
        }
        self.save_resolution(checkpoint)
    }

    pub(in crate::services::extensions) fn validate_replay(
        &self,
    ) -> Result<(), InstallInterruption> {
        let checkpoint = self.saved()?.ok_or(InstallInterruption::Failed)?;
        let Some(lock) = checkpoint.dependency_lock else {
            return Ok(());
        };
        if lock
            .relative_root
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return Err(InstallInterruption::Failed);
        }
        let staging = super::super::managed_store::root()
            .join(format!(".staging-{}", checkpoint.token))
            .canonicalize()
            .map_err(|_| InstallInterruption::Failed)?;
        let root = staging
            .join(&lock.relative_root)
            .canonicalize()
            .map_err(|_| InstallInterruption::Failed)?;
        if !root.starts_with(&staging) {
            return Err(InstallInterruption::Failed);
        }
        let actual = digest(&root)?;
        if !super::super::fingerprint::same_encoded(Some(&lock.digest), Some(&actual)) {
            return Err(InstallInterruption::Failed);
        }
        Ok(())
    }

    fn save_resolution(
        &self,
        checkpoint: super::checkpoint::InstallCheckpoint,
    ) -> Result<(), InstallInterruption> {
        let mut state = self.store.lock().map_err(|_| InstallInterruption::Failed)?;
        let index = state
            .index(&self.id)
            .map_err(|_| InstallInterruption::Failed)?;
        let current = state.jobs[index]
            .checkpoint
            .as_mut()
            .ok_or(InstallInterruption::Failed)?;
        current.dependency_lock = checkpoint.dependency_lock;
        current.resolved_source = checkpoint.resolved_source;
        if self.store.persist(&state).is_err() {
            state.durable_error = true;
            self.cancel.cancel();
            return Err(InstallInterruption::Failed);
        }
        Ok(())
    }
}

fn digest(root: &Path) -> Result<String, InstallInterruption> {
    let lock = LOCKS
        .iter()
        .map(|name| root.join(name))
        .find(|path| path.exists())
        .ok_or(InstallInterruption::Failed)?;
    let mut hash = Sha256::new();
    for path in [root.join(MANIFEST), lock] {
        let bytes = read(&path)?;
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    Ok(hex::encode(hash.finalize()))
}
fn read(path: &Path) -> Result<Vec<u8>, InstallInterruption> {
    match crate::services::private_store::read_bounded_regular(path, MAX_LOCK_BYTES)
        .map_err(|_| InstallInterruption::Failed)?
    {
        crate::services::private_store::BoundedFile::Content(bytes) => Ok(bytes),
        _ => Err(InstallInterruption::Failed),
    }
}
