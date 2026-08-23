#![allow(dead_code)]

use super::canonical_executable::CanonicalExecutable;
use super::error::OllamaErrorCode;
use super::path_identity::{CanonicalDirectory, PathIdentityResolver};
use super::spawn_profile_paths::{
    active_executable, overlaps, resolve_models_path, transaction_locations,
    verified_models_directory,
};
use super::types::OllamaEndpoint;
use crate::services::paths::OllamaPaths;
use std::ffi::OsString;
use std::path::Path;

#[allow(unused_imports)]
pub(crate) use super::constants::{
    MAX_OLLAMA_ENV_ENTRIES, MAX_OLLAMA_ENV_KEY_UNITS, MAX_OLLAMA_ENV_TOTAL_UNIX_BYTES,
    MAX_OLLAMA_ENV_TOTAL_WINDOWS_UTF16, MAX_OLLAMA_ENV_VALUE_UNITS,
};
pub(crate) use super::spawn_environment::FrozenEnvironment;

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
            super::spawn_environment::collect_inherited_bounded(inherited_environment)?,
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
            super::spawn_environment::collect_inherited_bounded(inherited_environment)?,
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
            super::spawn_environment::collect_inherited_bounded(inherited_environment)?,
            inherited_cwd,
            identity,
            false,
            super::spawn_environment::collect_bounded(overrides)?,
        )
    }

    pub(crate) fn validate_models_confinement(
        paths: &OllamaPaths,
        inherited_environment: impl IntoIterator<Item = (OsString, OsString)>,
        inherited_cwd: &Path,
        identity: &dyn PathIdentityResolver,
    ) -> Result<(), OllamaErrorCode> {
        let inherited = super::spawn_environment::collect_inherited_bounded(inherited_environment)?;
        let environment = super::spawn_environment::freeze(inherited, Vec::new())?;
        let working_directory = identity.canonical_directory(inherited_cwd)?;
        resolve_models_directory(paths, &environment, &working_directory, identity, false)
            .map(|_| ())
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
        let models_directory = resolve_models_directory(
            paths,
            &inherited_snapshot,
            &working_directory,
            identity,
            probe,
        )?;
        let active_directory = identity
            .verified_location(&paths.active)?
            .comparison_directory();
        let executable =
            identity.canonical_executable(&active_executable(active_directory.path()))?;
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

fn resolve_models_directory(
    paths: &OllamaPaths,
    environment: &FrozenEnvironment,
    working_directory: &CanonicalDirectory,
    identity: &dyn PathIdentityResolver,
    probe: bool,
) -> Result<CanonicalDirectory, OllamaErrorCode> {
    let models_path = if probe {
        paths.probe_models.clone()
    } else {
        resolve_models_path(
            environment.value("OLLAMA_MODELS"),
            working_directory,
            environment,
        )?
    };
    let models_directory = verified_models_directory(&models_path, identity)?;
    for transaction_path in transaction_locations(paths, probe) {
        let transaction = identity
            .verified_location(transaction_path)?
            .comparison_directory();
        if overlaps(identity, &models_directory, &transaction)? {
            return Err(OllamaErrorCode::OllamaModelStoreConflict);
        }
    }
    Ok(models_directory)
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
