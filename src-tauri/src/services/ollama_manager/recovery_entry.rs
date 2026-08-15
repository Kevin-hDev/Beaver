#![allow(dead_code)]

use super::durable_fs::platform_fs;
use super::error::OllamaErrorCode;
use super::manager::OllamaManager;
use super::path_identity::{CanonicalDirectory, NativePathIdentityResolver, PathIdentityResolver};
use super::probe::{OllamaTargetProbe, OwnedOllamaTargetProbe, PreparedBundle, TargetValidation};
use super::recovery::{
    RecoveryExecutor, RecoveryOutcome, RecoveryProbe, RecoveryProbeResult, RecoveryReason,
};
use super::spawn_profile::OllamaSpawnProfile;
use super::types::OperationState;
use crate::services::paths::{data_dir, ollama_paths};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

const RECOVERY_PROBE_TIMEOUT: Duration = Duration::from_secs(3);

struct PlatformRecoveryProbe;

#[async_trait::async_trait]
impl RecoveryProbe for PlatformRecoveryProbe {
    async fn validate(
        &self,
        target: &super::fingerprint::BundleFingerprint,
        paths: &crate::services::paths::OllamaPaths,
    ) -> RecoveryProbeResult {
        let identity = NativePathIdentityResolver;
        let cwd = match std::env::current_dir() {
            Ok(cwd) => cwd,
            Err(_) => {
                return RecoveryProbeResult::Deferred(OllamaErrorCode::OllamaStorageUnavailable)
            }
        };
        let profile =
            match OllamaSpawnProfile::resolve_probe(paths, std::env::vars_os(), &cwd, &identity) {
                Ok(profile) => profile,
                Err(code) => return RecoveryProbeResult::Deferred(code),
            };
        let root = match identity.canonical_directory(&paths.active) {
            Ok(root) => root,
            Err(code) => return RecoveryProbeResult::Deferred(code),
        };
        let prepared = PreparedBundle {
            root,
            executable: profile.executable().clone(),
            fingerprint: target.clone(),
        };
        let probe = OwnedOllamaTargetProbe::with_deadline(Instant::now() + RECOVERY_PROBE_TIMEOUT);
        let cancellation = CancellationToken::new();
        match probe.validate(&prepared, &profile, &cancellation).await {
            TargetValidation::Valid { .. } => RecoveryProbeResult::Valid,
            TargetValidation::InvalidTarget { code } => RecoveryProbeResult::Invalid(code),
            TargetValidation::Deferred { code } => RecoveryProbeResult::Deferred(code),
        }
    }
}

pub(crate) async fn recover_platform(
    reason: RecoveryReason,
) -> Result<RecoveryOutcome, OllamaErrorCode> {
    let paths = ollama_paths(&data_dir());
    let fs = Arc::new(platform_fs());
    let probe = Arc::new(PlatformRecoveryProbe);
    let executor = match frozen_models_directory(&paths) {
        Some(models) => RecoveryExecutor::new_with_models(fs, probe, paths, models),
        None => RecoveryExecutor::new(fs, probe, paths),
    };
    executor.recover(reason).await
}

fn frozen_models_directory(
    paths: &crate::services::paths::OllamaPaths,
) -> Option<CanonicalDirectory> {
    let cwd = std::env::current_dir().ok()?;
    match std::fs::symlink_metadata(&paths.active) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            if let Ok(profile) = OllamaSpawnProfile::resolve(
                paths,
                std::env::vars_os(),
                &cwd,
                &NativePathIdentityResolver,
            ) {
                return Some(profile.models_directory().clone());
            }
        }
        Ok(_) => return None,
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => return None,
        Err(_) => {}
    }
    let environment = super::spawn_environment::freeze(
        super::spawn_environment::collect_bounded(std::env::vars_os()).ok()?,
        Vec::new(),
    )
    .ok()?;
    let cwd = NativePathIdentityResolver.canonical_directory(&cwd).ok()?;
    let model_path = super::spawn_profile_paths::resolve_models_path(
        environment.value("OLLAMA_MODELS"),
        &cwd,
        &environment,
    )
    .ok()?;
    Some(
        NativePathIdentityResolver
            .verified_location(&model_path)
            .ok()?
            .comparison_directory(),
    )
}

impl OllamaManager {
    pub async fn recover(
        &self,
        reason: RecoveryReason,
    ) -> Result<RecoveryOutcome, OllamaErrorCode> {
        let guard = self.begin_operation(OperationState::Recovering).await?;
        let result = recover_platform(reason).await;
        match &result {
            Err(code) => guard.fail(*code),
            Ok(RecoveryOutcome::Deferred { code }) => guard.fail(*code),
            Ok(_) => drop(guard),
        }
        result
    }
}
