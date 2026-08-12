use super::model_downloads_store::{list_locked, DownloadStore, ModelDownloadManager};
use super::model_downloads_types::{ModelDownloadState, ModelDownloadStatus};
use std::time::Instant;
use tokio_util::sync::CancellationToken;

impl ModelDownloadManager {
    pub async fn complete_and_activate_next(
        &self,
    ) -> Option<(ModelDownloadState, CancellationToken)> {
        let mut store = self.inner.lock().await;
        let next = activate_next_locked(&mut store);
        set_worker_state(&mut store, next.is_some());
        next
    }

    pub async fn cancel(&self, id: &str) -> Result<Vec<ModelDownloadState>, String> {
        let mut store = self.inner.lock().await;
        let entry = store
            .entries
            .get_mut(id)
            .ok_or_else(|| "model-download-not-found".to_string())?;
        entry.cancel.cancel();
        if entry.state.status == ModelDownloadStatus::Queued {
            entry.state.status = ModelDownloadStatus::Cancelled;
        }
        Ok(list_locked(&store))
    }

    pub async fn cancel_all(&self) {
        let mut store = self.inner.lock().await;
        for entry in store.entries.values_mut() {
            entry.cancel.cancel();
            if entry.state.status == ModelDownloadStatus::Queued {
                entry.state.status = ModelDownloadStatus::Cancelled;
            }
        }
    }

    pub async fn worker_start_failed(&self, id: &str) {
        let mut store = self.inner.lock().await;
        if let Some(entry) = store.entries.get_mut(id) {
            entry.cancel.cancel();
            entry.state.status = ModelDownloadStatus::Cancelled;
        }
        set_worker_state(&mut store, false);
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.work.begin_closing();
        self.cancel_all().await;
        self.work.stop_and_wait(deadline).await
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
