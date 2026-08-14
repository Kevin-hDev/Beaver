use super::path_identity::{
    CanonicalDirectory, NativeDirectoryIdentity, PathIdentityResolver, ValidatedPathComponent,
    VerifiedDirectoryLocation,
};
use crate::services::paths::ollama_paths;
use std::collections::{HashMap, VecDeque};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const ROOT: &str = "/fake/data";
const MAX_FAKE_CALLS: usize = 256;

#[derive(Clone)]
pub struct FakeResolver {
    calls: Arc<Mutex<VecDeque<String>>>,
    directories: Arc<HashMap<PathBuf, CanonicalDirectory>>,
    pub locations: Arc<HashMap<PathBuf, VerifiedDirectoryLocation>>,
    failure: Option<super::error::OllamaErrorCode>,
    mutations: Arc<Mutex<usize>>,
}

impl FakeResolver {
    pub fn with_paths(paths: &crate::services::paths::OllamaPaths) -> Self {
        let mut directories = HashMap::new();
        directories.insert(PathBuf::from(ROOT), directory(ROOT, 1));
        directories.insert(PathBuf::from("/fake/cwd"), directory("/fake/cwd", 2));
        let mut locations = HashMap::new();
        for path in transaction_paths(paths) {
            locations.insert(path.clone(), absent_location(ROOT, path));
        }
        locations.insert(
            PathBuf::from("/fake/cwd/models"),
            absent_location("/fake/cwd", Path::new("/fake/cwd/models")),
        );
        locations.insert(
            PathBuf::from("/fake/data/.ollama/models"),
            absent_location("/fake/data/.ollama", Path::new("/fake/data/.ollama/models")),
        );
        locations.insert(
            PathBuf::from("/fake/home/.ollama/models"),
            absent_location("/fake/home/.ollama", Path::new("/fake/home/.ollama/models")),
        );
        locations.insert(
            paths.probe_models.clone(),
            absent_location(ROOT, &paths.probe_models),
        );
        Self {
            calls: Arc::new(Mutex::new(VecDeque::new())),
            directories: Arc::new(directories),
            locations: Arc::new(locations),
            failure: None,
            mutations: Arc::new(Mutex::new(0)),
        }
    }

    pub fn fail_with(mut self, code: super::error::OllamaErrorCode) -> Self {
        self.failure = Some(code);
        self
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls
            .lock()
            .expect("calls lock")
            .iter()
            .cloned()
            .collect()
    }

    fn record_call(&self, call: String) {
        let mut calls = self.calls.lock().expect("calls lock");
        assert!(calls.len() < MAX_FAKE_CALLS, "fake call file exhausted");
        calls.push_back(call);
    }

    pub fn mutation_count(&self) -> usize {
        *self.mutations.lock().expect("mutation lock")
    }
}

impl PathIdentityResolver for FakeResolver {
    fn canonical_directory(
        &self,
        path: &Path,
    ) -> Result<CanonicalDirectory, super::error::OllamaErrorCode> {
        self.record_call(format!("canonical:{}", path.display()));
        if let Some(error) = self.failure {
            return Err(error);
        }
        self.directories
            .get(path)
            .cloned()
            .ok_or(super::error::OllamaErrorCode::OllamaStorageUnavailable)
    }

    fn verified_location(
        &self,
        path: &Path,
    ) -> Result<VerifiedDirectoryLocation, super::error::OllamaErrorCode> {
        self.record_call(format!("verify:{}", path.display()));
        if let Some(error) = self.failure {
            return Err(error);
        }
        self.locations
            .get(path)
            .cloned()
            .ok_or(super::error::OllamaErrorCode::OllamaStorageUnavailable)
    }

    fn same_directory(
        &self,
        left: &CanonicalDirectory,
        right: &CanonicalDirectory,
    ) -> Result<bool, super::error::OllamaErrorCode> {
        self.record_call(format!(
            "same:{}:{}",
            left.path().display(),
            right.path().display()
        ));
        Ok(match (left.identity(), right.identity()) {
            (Some(left), Some(right)) => left == right,
            _ => left.path() == right.path(),
        })
    }

    fn contains(
        &self,
        parent: &CanonicalDirectory,
        child: &CanonicalDirectory,
    ) -> Result<bool, super::error::OllamaErrorCode> {
        self.record_call(format!(
            "contains:{}:{}",
            parent.path().display(),
            child.path().display()
        ));
        if parent.identity().is_some() && parent.identity() == child.identity() {
            return Ok(false);
        }
        Ok(child.path().starts_with(parent.path()))
    }
}

pub fn directory(path: &str, identity: u64) -> CanonicalDirectory {
    CanonicalDirectory::synthetic(
        PathBuf::from(path),
        Some(NativeDirectoryIdentity::synthetic(identity)),
    )
}

pub fn absent_location(parent: &str, path: &Path) -> VerifiedDirectoryLocation {
    let leaf = path
        .file_name()
        .and_then(OsStr::to_str)
        .map(ValidatedPathComponent::new)
        .expect("valid fake leaf");
    VerifiedDirectoryLocation::absent(directory(parent, 1), leaf)
}

pub fn existing_location(path: &str, identity: u64) -> VerifiedDirectoryLocation {
    VerifiedDirectoryLocation::existing(directory(path, identity))
}

pub fn transaction_paths(paths: &crate::services::paths::OllamaPaths) -> Vec<&PathBuf> {
    vec![
        &paths.active,
        &paths.legacy_staging,
        &paths.legacy_backup,
        &paths.failed,
        &paths.install_staging,
        &paths.update_staging,
        &paths.backup,
        &paths.backup_delete,
        &paths.failed_delete,
    ]
}

pub fn paths() -> crate::services::paths::OllamaPaths {
    ollama_paths(Path::new(ROOT))
}

pub fn env(entries: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    entries
        .iter()
        .map(|(key, value)| (OsString::from(key), OsString::from(value)))
        .collect()
}

pub fn resolve(
    resolver: &FakeResolver,
    inherited: &[(&str, &str)],
) -> Result<super::spawn_profile::OllamaSpawnProfile, super::error::OllamaErrorCode> {
    super::spawn_profile::OllamaSpawnProfile::resolve(
        &paths(),
        env(inherited),
        Path::new("/fake/cwd"),
        resolver,
    )
}
