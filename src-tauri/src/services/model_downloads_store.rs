use super::model_downloads_types::{
    ModelDownloadKind, ModelDownloadPhase, ModelDownloadState, ModelDownloadStatus,
    MAX_PENDING_DOWNLOADS,
};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ModelDownloadManager {
    inner: Arc<Mutex<DownloadStore>>,
}

#[derive(Debug, Default)]
struct DownloadStore {
    entries: HashMap<String, DownloadEntry>,
    order: VecDeque<String>,
    worker_running: bool,
}

#[derive(Debug, Clone)]
struct DownloadEntry {
    state: ModelDownloadState,
    cancel: CancellationToken,
}

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub phase: ModelDownloadPhase,
    pub downloaded: u64,
    pub total: u64,
    pub percent: u8,
}

impl ModelDownloadManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(DownloadStore::default())),
        }
    }

    pub fn inner_clone(&self) -> Self {
        self.clone()
    }

    pub async fn start(
        &self,
        kind: ModelDownloadKind,
        model_id: String,
        is_update: bool,
    ) -> Result<(ModelDownloadState, Option<CancellationToken>), String> {
        let mut store = self.inner.lock().await;
        remove_finished(&mut store);
        if store.entries.values().any(|entry| {
            entry.state.kind == kind
                && entry.state.model_id == model_id
                && is_pending(entry.state.status)
        }) {
            return Err("model-download-already-queued".into());
        }
        if store
            .entries
            .values()
            .filter(|entry| is_pending(entry.state.status))
            .count()
            >= MAX_PENDING_DOWNLOADS
        {
            return Err("model-download-queue-full".into());
        }

        let id = Uuid::new_v4().to_string();
        let cancel = CancellationToken::new();
        let runs_now = !store.worker_running;
        let status = if runs_now {
            store.worker_running = true;
            ModelDownloadStatus::Running
        } else {
            ModelDownloadStatus::Queued
        };
        let state = ModelDownloadState::new(kind, model_id, is_update, id.clone(), status);
        store.order.push_back(id.clone());
        store.entries.insert(
            id,
            DownloadEntry {
                state: state.clone(),
                cancel: cancel.clone(),
            },
        );
        Ok((state, runs_now.then_some(cancel)))
    }

    pub async fn list(&self) -> Vec<ModelDownloadState> {
        let store = self.inner.lock().await;
        list_locked(&store)
    }

    pub async fn progress(&self, id: &str, update: ProgressUpdate) -> Vec<ModelDownloadState> {
        let mut store = self.inner.lock().await;
        if let Some(entry) = store.entries.get_mut(id) {
            entry.state.phase = update.phase;
            entry.state.downloaded = update.downloaded;
            entry.state.total = update.total;
            entry.state.percent = update.percent.min(100);
        }
        list_locked(&store)
    }

    pub async fn finish(
        &self,
        id: &str,
        status: ModelDownloadStatus,
        error_key: Option<&str>,
    ) -> Vec<ModelDownloadState> {
        let mut store = self.inner.lock().await;
        if let Some(entry) = store.entries.get_mut(id) {
            entry.state.status = status;
            entry.state.error_key = error_key.map(str::to_string);
            if status == ModelDownloadStatus::Completed {
                entry.state.phase = ModelDownloadPhase::Completed;
                entry.state.percent = 100;
            }
        }
        list_locked(&store)
    }

    pub async fn activate_next(&self) -> Option<(ModelDownloadState, CancellationToken)> {
        let mut store = self.inner.lock().await;
        let next_id = store
            .order
            .iter()
            .find(|id| {
                store
                    .entries
                    .get(*id)
                    .is_some_and(|entry| entry.state.status == ModelDownloadStatus::Queued)
            })
            .cloned();
        let Some(next_id) = next_id else {
            store.worker_running = false;
            return None;
        };
        let entry = store.entries.get_mut(&next_id)?;
        entry.state.status = ModelDownloadStatus::Running;
        Some((entry.state.clone(), entry.cancel.clone()))
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
}

fn is_pending(status: ModelDownloadStatus) -> bool {
    matches!(
        status,
        ModelDownloadStatus::Queued | ModelDownloadStatus::Running
    )
}

fn remove_finished(store: &mut DownloadStore) {
    store
        .entries
        .retain(|_, entry| is_pending(entry.state.status));
    store.order.retain(|id| store.entries.contains_key(id));
}

fn list_locked(store: &DownloadStore) -> Vec<ModelDownloadState> {
    store
        .order
        .iter()
        .filter_map(|id| store.entries.get(id).map(|entry| entry.state.clone()))
        .collect()
}

impl Default for ModelDownloadManager {
    fn default() -> Self {
        Self::new()
    }
}
