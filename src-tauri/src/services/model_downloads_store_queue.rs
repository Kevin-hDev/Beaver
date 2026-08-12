use super::model_downloads_store::{list_locked, DownloadStore, ModelDownloadManager};
use super::model_downloads_types::{ModelDownloadState, ModelDownloadStatus};
use crate::services::work_registry::ServiceWorkCancellation;
use std::time::Instant;
use tokio_util::sync::CancellationToken;

impl ModelDownloadManager {
    #[cfg(test)]
    pub async fn activate_next(&self) -> Option<(ModelDownloadState, CancellationToken)> {
        let mut store = self.inner.lock().await;
        let next = activate_next_locked(&mut store);
        if next.is_none() {
            store.worker_running = false;
        }
        next
    }

    pub async fn wait_for_next(
        &self,
        shutdown: &ServiceWorkCancellation,
    ) -> Option<(ModelDownloadState, CancellationToken)> {
        loop {
            if shutdown.is_cancelled() {
                return None;
            }
            let changed = self.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            let mut store = self.inner.lock().await;
            if let Some(next) = activate_next_locked(&mut store) {
                return Some(next);
            }
            drop(store);
            tokio::select! {
                _ = changed => {}
                _ = shutdown.cancelled() => return None,
            }
        }
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
        drop(store);
        self.changed.notify_waiters();
    }

    pub async fn worker_start_failed(&self, id: &str) {
        let mut store = self.inner.lock().await;
        if let Some(entry) = store.entries.get_mut(id) {
            entry.cancel.cancel();
            entry.state.status = ModelDownloadStatus::Cancelled;
        }
        store.worker_running = false;
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.work.begin_closing();
        self.cancel_all().await;
        self.work.stop_and_wait(deadline).await
    }
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
