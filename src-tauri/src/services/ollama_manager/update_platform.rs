use super::super::blocking::run_ollama_blocking;
use super::super::cleanup_inspection;
use super::super::durable_fs::{platform_fs, OllamaDurableFs, PlatformOllamaDurableFs};
use super::super::error::OllamaErrorCode;
use super::super::fingerprint::BundleFingerprint;
use super::super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::super::journal_store::OllamaJournalStore;
use super::super::path_identity::NativePathIdentityResolver;
use super::super::probe::{
    OllamaTargetProbe, OwnedOllamaTargetProbe, PreparedBundle, TargetValidation,
};
use super::super::recovery_decision::{DirectoryEvidence, JournalPresence};
use super::super::spawn_profile::OllamaSpawnProfile;
use super::{
    execute, CompletionRecovery, UpdateBackend, UpdateOutcome, UpdateRequest, UpdateSidecar,
};
use crate::services::paths::OllamaPaths;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[path = "update_platform_completion.rs"]
mod completion;
#[path = "update_platform_preflight.rs"]
mod preflight;
#[path = "update_platform_prepare.rs"]
mod prepare;

const UPDATE_PROBE_TIMEOUT: Duration = Duration::from_secs(30);

struct PlatformUpdateBackend {
    paths: OllamaPaths,
    fs: Arc<PlatformOllamaDurableFs>,
    journal: OllamaJournalStore<PlatformOllamaDurableFs>,
}

impl PlatformUpdateBackend {
    fn new(paths: OllamaPaths) -> Self {
        let fs = Arc::new(platform_fs());
        let journal = OllamaJournalStore::new(Arc::clone(&fs), paths.clone());
        Self { paths, fs, journal }
    }

    async fn rename(
        &self,
        source: &std::path::Path,
        destination: &std::path::Path,
    ) -> Result<(), OllamaErrorCode> {
        let fs = Arc::clone(&self.fs);
        let source = source.to_path_buf();
        let destination = destination.to_path_buf();
        run_ollama_blocking(move || {
            fs.rename_durable(&source, &destination).map_err(|error| {
                super::super::storage_error::durable("update-bundle-rename", error)
            })
        })
        .await
    }
}

#[async_trait::async_trait]
impl UpdateBackend for PlatformUpdateBackend {
    async fn journal(&self) -> Result<Option<OllamaTransactionJournal>, OllamaErrorCode> {
        self.journal.read().await
    }

    async fn current(&self) -> Result<BundleFingerprint, OllamaErrorCode> {
        match cleanup_inspection::snapshot(JournalPresence::Absent, &*self.fs, &self.paths).active {
            DirectoryEvidence::Present(fingerprint) => Ok(fingerprint),
            DirectoryEvidence::Absent => Err(OllamaErrorCode::OllamaBundleMissing),
            DirectoryEvidence::Invalid => Err(OllamaErrorCode::OllamaBundleInvalid),
            DirectoryEvidence::Incomplete | DirectoryEvidence::Unknown => {
                Err(OllamaErrorCode::OllamaStorageUnavailable)
            }
        }
    }

    async fn prepare_target(
        &self,
        request: &UpdateRequest,
    ) -> Result<PreparedBundle, OllamaErrorCode> {
        prepare::prepare(self, request).await
    }

    async fn persist(
        &self,
        state: OllamaJournalState,
        replace: bool,
    ) -> Result<(), OllamaErrorCode> {
        let journal = OllamaTransactionJournal::new(state);
        if replace {
            self.journal.replace(&journal).await
        } else {
            self.journal.write_new(&journal).await
        }
    }

    async fn stop_owned_sidecar(&self, request: &UpdateRequest) -> Result<(), OllamaErrorCode> {
        match &request.sidecar {
            UpdateSidecar::Owned(sidecar) => sidecar.stop(),
            UpdateSidecar::Absent | UpdateSidecar::External => Ok(()),
        }
    }

    async fn reap_owned_sidecar(&self, request: &UpdateRequest) -> Result<(), OllamaErrorCode> {
        match &request.sidecar {
            UpdateSidecar::Owned(sidecar) => sidecar.reap(),
            UpdateSidecar::Absent | UpdateSidecar::External => Ok(()),
        }
    }

    async fn rename_active_to_backup(&self) -> Result<(), OllamaErrorCode> {
        self.rename(&self.paths.active, &self.paths.backup).await
    }

    async fn rename_target_to_active(&self) -> Result<(), OllamaErrorCode> {
        self.rename(&self.paths.update_staging, &self.paths.active)
            .await
    }

    async fn probe_active(
        &self,
        request: &UpdateRequest,
        target: &PreparedBundle,
    ) -> TargetValidation {
        let receipt_path = crate::services::paths::bundle_receipt_path(&request.paths.active);
        match super::super::bundle_receipt::read_receipt(&*self.fs, &receipt_path) {
            Ok(Some(receipt)) if receipt.fingerprint == target.fingerprint => {}
            Ok(Some(_)) | Ok(None) => {
                return TargetValidation::InvalidTarget {
                    code: OllamaErrorCode::OllamaBundleInvalid,
                }
            }
            Err(code) => return TargetValidation::Deferred { code },
        }
        let profile = match OllamaSpawnProfile::resolve_probe(
            &request.paths,
            request.inherited_environment.clone(),
            &request.inherited_cwd,
            &NativePathIdentityResolver,
        ) {
            Ok(profile) => profile,
            Err(OllamaErrorCode::OllamaBundleInvalid) => {
                return TargetValidation::InvalidTarget {
                    code: OllamaErrorCode::OllamaBundleInvalid,
                }
            }
            Err(OllamaErrorCode::OllamaModelStoreConflict) => {
                return TargetValidation::Deferred {
                    code: OllamaErrorCode::OllamaModelStoreConflict,
                }
            }
            Err(code) => return TargetValidation::Deferred { code },
        };
        let deadline = request
            .deadline
            .unwrap_or_else(|| Instant::now() + UPDATE_PROBE_TIMEOUT);
        OwnedOllamaTargetProbe::with_deadline(deadline)
            .validate(target, &profile, &request.cancellation)
            .await
    }

    async fn recover_completion(&self) -> Result<CompletionRecovery, OllamaErrorCode> {
        completion::recover().await
    }
}

pub(crate) async fn run(request: UpdateRequest) -> Result<UpdateOutcome, OllamaErrorCode> {
    preflight::validate_request(&request)?;
    if request.manifest.is_none() {
        return Err(OllamaErrorCode::OllamaDownloadFailed);
    }
    let backend = PlatformUpdateBackend::new(request.paths.clone());
    execute(&backend, &request).await
}
