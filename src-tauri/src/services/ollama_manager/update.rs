use super::error::OllamaErrorCode;
use super::fingerprint::{BundleFingerprint, OllamaVersion};
use super::release_source::OllamaReleaseManifest;
#[cfg(test)]
use crate::services::paths::ollama_paths;
use crate::services::paths::OllamaPaths;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio_util::sync::CancellationToken;

#[path = "update_platform.rs"]
pub(crate) mod platform;

pub trait OwnedSidecarController: Send + Sync {
    fn stop(&self) -> Result<(), OllamaErrorCode>;
    fn reap(&self) -> Result<(), OllamaErrorCode>;
}

#[derive(Clone, Default)]
pub enum UpdateSidecar {
    #[default]
    Absent,
    External,
    Owned(Arc<dyn OwnedSidecarController>),
}

#[derive(Clone)]
pub struct UpdateRequest {
    pub paths: OllamaPaths,
    pub version: OllamaVersion,
    pub manifest: Option<OllamaReleaseManifest>,
    pub inherited_environment: Vec<(OsString, OsString)>,
    pub inherited_cwd: PathBuf,
    pub cancellation: CancellationToken,
    pub deadline: Option<Instant>,
    pub sidecar: UpdateSidecar,
}

impl UpdateRequest {
    #[cfg(test)]
    pub(crate) fn for_test(root: PathBuf) -> Self {
        Self {
            paths: ollama_paths(&root),
            version: OllamaVersion::parse("1.2.3").expect("test version"),
            manifest: None,
            inherited_environment: Vec::new(),
            inherited_cwd: root.clone(),
            cancellation: CancellationToken::new(),
            deadline: None,
            sidecar: UpdateSidecar::Absent,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdateOutcome {
    #[allow(dead_code)]
    Updated {
        fingerprint: BundleFingerprint,
    },
    AlreadyCurrent,
    CleanupPending {
        code: OllamaErrorCode,
    },
    Deferred {
        code: OllamaErrorCode,
    },
}

#[async_trait::async_trait]
pub(crate) trait UpdateBackend: Send + Sync {
    async fn journal(
        &self,
    ) -> Result<Option<super::journal::OllamaTransactionJournal>, OllamaErrorCode>;
    async fn current(&self) -> Result<BundleFingerprint, OllamaErrorCode>;
    async fn prepare_target(
        &self,
        request: &UpdateRequest,
    ) -> Result<super::probe::PreparedBundle, OllamaErrorCode>;
    async fn persist(
        &self,
        state: super::journal::OllamaJournalState,
        replace: bool,
    ) -> Result<(), OllamaErrorCode>;
    async fn stop_owned_sidecar(&self, request: &UpdateRequest) -> Result<(), OllamaErrorCode>;
    async fn reap_owned_sidecar(&self, request: &UpdateRequest) -> Result<(), OllamaErrorCode>;
    async fn rename_active_to_backup(&self) -> Result<(), OllamaErrorCode>;
    async fn rename_target_to_active(&self) -> Result<(), OllamaErrorCode>;
    async fn probe_active(
        &self,
        request: &UpdateRequest,
        target: &super::probe::PreparedBundle,
    ) -> super::probe::TargetValidation;
}

pub(crate) async fn execute<B: UpdateBackend>(
    backend: &B,
    request: &UpdateRequest,
) -> Result<UpdateOutcome, OllamaErrorCode> {
    if request.cancellation.is_cancelled() {
        return Err(OllamaErrorCode::OllamaOperationCancelled);
    }
    if matches!(request.sidecar, UpdateSidecar::External) {
        return Ok(UpdateOutcome::Deferred {
            code: OllamaErrorCode::OllamaValidationDeferred,
        });
    }
    if let Some(journal) = backend.journal().await? {
        return match journal.state {
            super::journal::OllamaJournalState::CleanupPending { .. } => {
                Ok(UpdateOutcome::CleanupPending {
                    code: OllamaErrorCode::OllamaUpdateCleanupPending,
                })
            }
            _ => Err(OllamaErrorCode::OllamaUpdateRecoveryRequired),
        };
    }
    let previous = backend.current().await?;
    let target = backend.prepare_target(request).await?;
    if target.fingerprint == previous {
        return Ok(UpdateOutcome::AlreadyCurrent);
    }
    backend
        .persist(
            super::journal::OllamaJournalState::Prepared {
                target: target.fingerprint.clone(),
                previous: previous.clone(),
            },
            false,
        )
        .await?;
    backend.stop_owned_sidecar(request).await?;
    backend.reap_owned_sidecar(request).await?;
    backend.rename_active_to_backup().await?;
    backend.rename_target_to_active().await?;
    backend
        .persist(
            super::journal::OllamaJournalState::PendingValidation {
                target: target.fingerprint.clone(),
                previous: previous.clone(),
            },
            true,
        )
        .await?;
    let validation = backend.probe_active(request, &target).await;
    match validation {
        super::probe::TargetValidation::Valid { fingerprint } => {
            backend
                .persist(
                    super::journal::OllamaJournalState::CleanupPending {
                        target: fingerprint.clone(),
                        previous: previous.clone(),
                    },
                    true,
                )
                .await?;
            Ok(UpdateOutcome::CleanupPending {
                code: OllamaErrorCode::OllamaUpdateCleanupPending,
            })
        }
        super::probe::TargetValidation::InvalidTarget { code } => {
            backend
                .persist(
                    super::journal::OllamaJournalState::RollbackPending {
                        previous: previous.clone(),
                        rejected_target: Some(target.fingerprint.clone()),
                    },
                    true,
                )
                .await?;
            Ok(UpdateOutcome::Deferred { code })
        }
        super::probe::TargetValidation::Deferred { code } => Ok(UpdateOutcome::Deferred { code }),
    }
}

pub(crate) async fn run(_request: UpdateRequest) -> Result<UpdateOutcome, OllamaErrorCode> {
    platform::run(_request).await
}
