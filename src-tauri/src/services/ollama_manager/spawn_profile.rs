#![allow(dead_code)]

use super::error::OllamaErrorCode;
use super::path_identity::{CanonicalDirectory, PathIdentityResolver};
use super::spawn_profile_paths::{
    active_executable, overlaps, resolve_models_path, transaction_locations,
};
use super::types::OllamaEndpoint;
use crate::services::paths::OllamaPaths;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[allow(unused_imports)]
pub(crate) use super::constants::{
    MAX_OLLAMA_ENV_ENTRIES, MAX_OLLAMA_ENV_KEY_UNITS, MAX_OLLAMA_ENV_TOTAL_UNIX_BYTES,
    MAX_OLLAMA_ENV_TOTAL_WINDOWS_UTF16, MAX_OLLAMA_ENV_VALUE_UNITS,
};
pub(crate) use super::spawn_environment::FrozenEnvironment;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalExecutable {
    path: PathBuf,
}

impl CanonicalExecutable {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OllamaSpawnProfile {
    executable: CanonicalExecutable,
    working_directory: CanonicalDirectory,
    models_directory: CanonicalDirectory,
    environment: FrozenEnvironment,
}

impl OllamaSpawnProfile {
    pub fn resolve(
        paths: &OllamaPaths,
        inherited_environment: impl IntoIterator<Item = (OsString, OsString)>,
        inherited_cwd: &Path,
        identity: &dyn PathIdentityResolver,
    ) -> Result<Self, OllamaErrorCode> {
        Self::resolve_inner(
            paths,
            super::spawn_environment::collect_bounded(inherited_environment)?,
            inherited_cwd,
            identity,
            false,
            Vec::new(),
        )
    }

    pub fn resolve_probe(
        paths: &OllamaPaths,
        inherited_environment: impl IntoIterator<Item = (OsString, OsString)>,
        inherited_cwd: &Path,
        identity: &dyn PathIdentityResolver,
    ) -> Result<Self, OllamaErrorCode> {
        Self::resolve_inner(
            paths,
            super::spawn_environment::collect_bounded(inherited_environment)?,
            inherited_cwd,
            identity,
            true,
            Vec::new(),
        )
    }

    pub(crate) fn resolve_with_overrides(
        paths: &OllamaPaths,
        inherited_environment: impl IntoIterator<Item = (OsString, OsString)>,
        inherited_cwd: &Path,
        identity: &dyn PathIdentityResolver,
        overrides: impl IntoIterator<Item = (OsString, OsString)>,
    ) -> Result<Self, OllamaErrorCode> {
        Self::resolve_inner(
            paths,
            super::spawn_environment::collect_bounded(inherited_environment)?,
            inherited_cwd,
            identity,
            false,
            super::spawn_environment::collect_bounded(overrides)?,
        )
    }

    fn resolve_inner(
        paths: &OllamaPaths,
        inherited: Vec<(OsString, OsString)>,
        inherited_cwd: &Path,
        identity: &dyn PathIdentityResolver,
        probe: bool,
        dynamic_overrides: Vec<(OsString, OsString)>,
    ) -> Result<Self, OllamaErrorCode> {
        let inherited_snapshot = super::spawn_environment::freeze(inherited, Vec::new())?;
        let working_directory = identity.canonical_directory(inherited_cwd)?;
        let models_path = if probe {
            paths.probe_models.clone()
        } else {
            resolve_models_path(
                inherited_snapshot.value("OLLAMA_MODELS"),
                &working_directory,
                &inherited_snapshot,
            )?
        };
        let models = identity.verified_location(&models_path)?;
        let models_directory = models.comparison_directory();
        let active_directory = identity
            .verified_location(&paths.active)?
            .comparison_directory();
        for transaction_path in transaction_locations(paths, probe) {
            if transaction_path == &paths.active {
                continue;
            }
            let transaction = identity
                .verified_location(transaction_path)?
                .comparison_directory();
            if overlaps(identity, &models_directory, &transaction)? {
                return Err(OllamaErrorCode::OllamaModelStoreConflict);
            }
        }
        if overlaps(identity, &models_directory, &active_directory)? {
            return Err(OllamaErrorCode::OllamaModelStoreConflict);
        }
        let executable = CanonicalExecutable {
            path: active_executable(active_directory.path()),
        };
        let mut overrides = dynamic_overrides;
        overrides.push((
            OsString::from("OLLAMA_MODELS"),
            models_directory.path().as_os_str().to_owned(),
        ));
        overrides.push((OsString::from("OLLAMA_NO_CLOUD"), OsString::from("1")));
        let environment =
            super::spawn_environment::freeze_from_snapshot(inherited_snapshot, overrides)?;
        Ok(Self {
            executable,
            working_directory,
            models_directory,
            environment,
        })
    }

    pub(crate) fn executable(&self) -> &CanonicalExecutable {
        &self.executable
    }
    pub(crate) fn working_directory(&self) -> &CanonicalDirectory {
        &self.working_directory
    }
    pub(crate) fn models_directory(&self) -> &CanonicalDirectory {
        &self.models_directory
    }
    pub(crate) fn environment(&self) -> &FrozenEnvironment {
        &self.environment
    }
}

pub struct OllamaSpawnAttempt<'a> {
    profile: &'a OllamaSpawnProfile,
    endpoint: OllamaEndpoint,
}

impl<'a> OllamaSpawnAttempt<'a> {
    pub(crate) fn new(profile: &'a OllamaSpawnProfile, endpoint: OllamaEndpoint) -> Self {
        Self { profile, endpoint }
    }
    pub(crate) fn profile(&self) -> &'a OllamaSpawnProfile {
        self.profile
    }
    pub(crate) fn endpoint(&self) -> &OllamaEndpoint {
        &self.endpoint
    }
    pub(crate) fn port(&self) -> u16 {
        self.endpoint.port()
    }
}
