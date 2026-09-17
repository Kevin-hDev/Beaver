//! Startup never launches an installation. Only fully prepared sources may resume.
use super::{checkpoint, InstallJobStore, InstallPhase, InstallStatus};

impl InstallJobStore {
    pub(in crate::services::extensions) fn production(
        work: super::super::work_supervision::ExtensionWorkServices,
        app: tauri::AppHandle,
    ) -> Self {
        let executor = super::executor::ProductionExecutor::resolve(&app).ok();
        let store = Self::new(work, executor, Some(app));
        store.restore(checkpoint::path())
    }

    pub(super) fn restore(mut self, path: std::path::PathBuf) -> Self {
        self.journal = Some(path.clone());
        let result = checkpoint::load(&path);
        let mut state = self.state.lock().expect("new job store mutex");
        match result {
            Ok(Some(journal)) => {
                state.revision = journal.revision + 1;
                state.jobs = journal.jobs;
                let revision = state.revision;
                for job in &mut state.jobs {
                    if !job.view.status.terminal() {
                        job.view.status = InstallStatus::Interrupted;
                        job.finished_revision = Some(revision);
                    }
                    job.view.revision = revision;
                    job.view.confirmation_id = None;
                    job.view.can_cancel = false;
                    job.view.can_resume = false;
                    job.view.queue_blocker = None;
                    if job.checkpoint.is_none() {
                        job.clean = true;
                    }
                    if let Some(checkpoint) = &mut job.checkpoint {
                        // A persisted native identity means the gate may have opened.
                        // Keep the job blocked until explicit recovery confirms its scope.
                        if checkpoint.version == checkpoint::FORMAT {
                            checkpoint.producer_active = false;
                        }
                        if checkpoint.native_process.is_none() {
                            let published = checkpoint.record.as_ref().is_some_and(|record| {
                                super::super::registry::find(&record.manifest.id).is_ok_and(
                                    |installed| {
                                        installed.source == record.source
                                            && super::super::fingerprint::same_encoded(
                                                installed.fingerprint.as_deref(),
                                                record.fingerprint.as_deref(),
                                            )
                                    },
                                )
                            });
                            if published && job.view.status == InstallStatus::Interrupted {
                                job.view.status = InstallStatus::Completed;
                                job.view.extension_id = checkpoint
                                    .record
                                    .as_ref()
                                    .map(|record| record.manifest.id.clone());
                            } else if job.view.status == InstallStatus::Interrupted {
                                job.view.can_resume = resumable(checkpoint);
                            }
                        }
                    }
                }
                // Newly interrupted jobs survive ahead of older clean history.
                // Never discard owned artifacts merely to satisfy the UI bound.
                if state.evict(super::limits::MAX_RECENT).is_err() {
                    state.durable_error = true;
                    state.recovery_error = true;
                } else if self.persist(&state).is_err() {
                    state.durable_error = true;
                }
            }
            Ok(None) => {}
            Err(_) => {
                state.durable_error = true;
                state.recovery_error = true;
                log::warn!("extension install recovery journal unavailable");
            }
        }
        drop(state);
        self
    }
}

pub(super) fn resumable(checkpoint: &checkpoint::InstallCheckpoint) -> bool {
    checkpoint.version == checkpoint::FORMAT
        && !checkpoint.cleanup_unconfirmed
        && checkpoint.native_process.is_none()
        && checkpoint.safe_phase == Some(InstallPhase::BuildingUi)
        && checkpoint.record.as_ref().is_some_and(|record| {
            super::super::fingerprint::is_current(record).unwrap_or(false)
                && super::super::validation::records(std::slice::from_ref(record)).is_ok()
        })
}

pub(super) fn stop_recovered(checkpoint: &mut checkpoint::InstallCheckpoint) -> Result<(), String> {
    use crate::services::owned_process::OwnedProcess;
    if checkpoint.version != checkpoint::FORMAT
        && checkpoint.producer_active
        && checkpoint.native_process.is_none()
    {
        return Err(super::limits::UNAVAILABLE.into());
    }
    if let Some(identity) = checkpoint.native_process {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        if OwnedProcess::process_exists(identity.pid) {
            OwnedProcess::recover_exact(identity, deadline)
                .map_err(|_| super::limits::UNAVAILABLE)?;
        }
        #[cfg(unix)]
        {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|_| super::limits::UNAVAILABLE)?;
            if !runtime.block_on(
                crate::services::process_tree::confirm_recovered_group_absent(
                    identity.pid,
                    deadline,
                ),
            ) {
                return Err(super::limits::UNAVAILABLE.into());
            }
        }
        #[cfg(windows)]
        if OwnedProcess::process_exists(identity.pid) {
            return Err(super::limits::UNAVAILABLE.into());
        }
        checkpoint.native_process = None;
    }
    checkpoint.producer_active = false;
    checkpoint.cleanup_unconfirmed = false;
    Ok(())
}
