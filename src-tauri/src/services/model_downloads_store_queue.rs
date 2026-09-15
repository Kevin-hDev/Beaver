use super::model_downloads_store::{list_locked, DownloadStore, ModelDownloadManager};
use super::model_downloads_types::{ModelDownloadState, ModelDownloadStatus};
use std::time::Instant;
use tokio_util::sync::CancellationToken;

impl ModelDownloadManager {
    pub async fn complete_and_activate_next(
        &self,
    ) -> Option<(ModelDownloadState, CancellationToken)> {
        let mut store = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        let next = activate_next_locked(&mut store);
        set_worker_state(&mut store, next.is_some());
        next
    }

    pub async fn cancel(&self, id: &str) -> Result<Vec<ModelDownloadState>, String> {
        let mut store = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        let entry = store
            .entries
            .get_mut(id)
            .ok_or_else(|| "model-download-not-found".to_string())?;
        if matches!(
            entry.state.kind,
            super::model_downloads_types::ModelDownloadKind::Forecast
                | super::model_downloads_types::ModelDownloadKind::Voice
        ) && entry.state.phase == super::model_downloads_types::ModelDownloadPhase::Installing
        {
            return Ok(list_locked(&store));
        }
        entry.cancel.cancel();
        if entry.state.status == ModelDownloadStatus::Queued {
            entry.state.status = ModelDownloadStatus::Cancelled;
        } else if entry.state.status == ModelDownloadStatus::Running {
            entry.state.status = ModelDownloadStatus::Cancelling;
        } else if entry.state.status == ModelDownloadStatus::Suspended {
            entry.state.status = ModelDownloadStatus::Cancelled;
        }
        Ok(list_locked(&store))
    }

    pub async fn resume(
        &self,
        id: &str,
    ) -> Result<
        (
            ModelDownloadState,
            Option<(
                CancellationToken,
                super::model_downloads_store::DownloadWorkAdmission,
            )>,
        ),
        String,
    > {
        let mut store = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        if store
            .entries
            .get(id)
            .is_none_or(|entry| entry.state.status != ModelDownloadStatus::Suspended)
        {
            return Err("model-download-not-suspended".into());
        }
        let admission = if store.worker_running {
            self.work
                .try_probe()
                .map_err(|error| error.public_code().to_string())?;
            None
        } else {
            Some(
                self.work
                    .try_admit()
                    .map_err(|error| error.public_code().to_string())?,
            )
        };
        let runs_now = !store.worker_running;
        let entry = store.entries.get_mut(id).expect("checked suspended entry");
        entry.cancel = CancellationToken::new();
        entry.state.status = if runs_now {
            ModelDownloadStatus::Running
        } else {
            ModelDownloadStatus::Queued
        };
        let state = entry.state.clone();
        let cancel = entry.cancel.clone();
        if runs_now {
            store.worker_running = true;
        }
        Ok((state, admission.map(|admission| (cancel, admission))))
    }

    #[cfg(test)]
    pub async fn cancel_all(&self) {
        let mut store = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        for entry in store.entries.values_mut() {
            entry.cancel.cancel();
            if entry.state.status == ModelDownloadStatus::Queued {
                entry.state.status = ModelDownloadStatus::Cancelled;
            } else if entry.state.status == ModelDownloadStatus::Running {
                entry.state.status = ModelDownloadStatus::Cancelling;
            }
        }
    }

    pub async fn worker_start_failed(&self, id: &str) {
        let mut store = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(entry) = store.entries.get_mut(id) {
            entry.cancel.cancel();
            entry.state.status = ModelDownloadStatus::Cancelled;
        }
        set_worker_state(&mut store, false);
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.work.begin_closing();
        self.suspend_voice_and_cancel_others().await;
        self.work.stop_and_wait(deadline).await
    }

    async fn suspend_voice_and_cancel_others(&self) {
        let mut store = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        for entry in store.entries.values_mut() {
            entry.cancel.cancel();
            entry.state.status = if entry.state.kind
                == super::model_downloads_types::ModelDownloadKind::Voice
                && matches!(
                    entry.state.status,
                    ModelDownloadStatus::Queued | ModelDownloadStatus::Running
                ) {
                ModelDownloadStatus::Suspended
            } else if entry.state.status == ModelDownloadStatus::Queued {
                ModelDownloadStatus::Cancelled
            } else if entry.state.status == ModelDownloadStatus::Running {
                ModelDownloadStatus::Cancelling
            } else {
                entry.state.status
            };
        }
    }
}

// This function is the sole authority for worker_running transitions after
// admission; completion and startup failure cannot diverge on reset semantics.
fn set_worker_state(store: &mut DownloadStore, running: bool) {
    store.worker_running = running;
}

fn activate_next_locked(
    store: &mut DownloadStore,
) -> Option<(ModelDownloadState, CancellationToken)> {
    let next_id = store
        .order
        .iter()
        .find(|id| {
            store
                .entries
                .get(*id)
                .is_some_and(|entry| entry.state.status == ModelDownloadStatus::Queued)
        })
        .cloned()?;
    let entry = store.entries.get_mut(&next_id)?;
    entry.state.status = ModelDownloadStatus::Running;
    Some((entry.state.clone(), entry.cancel.clone()))
}
