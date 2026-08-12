use super::model_downloads_types::{
    ModelDownloadKind, ModelDownloadPhase, ModelDownloadState, ModelDownloadStatus,
    MAX_PENDING_DOWNLOADS,
};
use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::{ServiceWorkAdmission, ServiceWorkSupervisor};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const DOWNLOAD_WORKERS: usize = 1;
pub type DownloadWorkAdmission = ServiceWorkAdmission<DOWNLOAD_WORKERS>;

#[derive(Clone)]
pub struct ModelDownloadManager {
    pub(super) inner: Arc<Mutex<DownloadStore>>,
    pub(super) work: ServiceWorkSupervisor<DOWNLOAD_WORKERS>,
}

#[derive(Debug, Default)]
pub(super) struct DownloadStore {
    pub(super) entries: HashMap<String, DownloadEntry>,
    pub(super) order: VecDeque<String>,
    pub(super) worker_running: bool,
}

#[derive(Debug, Clone)]
pub(super) struct DownloadEntry {
    pub(super) state: ModelDownloadState,
    pub(super) cancel: CancellationToken,
}

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub phase: ModelDownloadPhase,
    pub downloaded: u64,
    pub total: u64,
    pub percent: u8,
}

impl ModelDownloadManager {
    pub fn new(app_work: AppWorkSupervisor) -> Self {
        Self {
            inner: Arc::new(Mutex::new(DownloadStore::default())),
            work: ServiceWorkSupervisor::new(app_work),
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
    ) -> Result<
        (
            ModelDownloadState,
            Option<(CancellationToken, DownloadWorkAdmission)>,
        ),
        String,
    > {
        let mut store = self.inner.lock().await;
        let admission = if store.worker_running {
            self.work.try_probe().map_err(public_admission_error)?;
            None
        } else {
            Some(self.work.try_admit().map_err(public_admission_error)?)
        };
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
        drop(store);
        Ok((state, admission.map(|admission| (cancel, admission))))
    }

    pub async fn list(&self) -> Vec<ModelDownloadState> {
        let store = self.inner.lock().await;
        list_locked(&store)
    }

    #[cfg(test)]
    pub async fn progress(&self, id: &str, update: ProgressUpdate) -> Vec<ModelDownloadState> {
        let mut store = self.inner.lock().await;
        apply_progress(&mut store, id, update);
        list_locked(&store)
    }

    pub fn try_progress(
        &self,
        id: &str,
        update: ProgressUpdate,
    ) -> Option<Vec<ModelDownloadState>> {
        // La progression est indicative : ne jamais bloquer un thread de téléchargement
        // pour une mise à jour d'interface qui sera remplacée par la suivante.
        let mut store = self.inner.try_lock().ok()?;
        apply_progress(&mut store, id, update);
        Some(list_locked(&store))
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
}

fn public_admission_error(
    error: crate::services::work_registry::ServiceWorkAdmissionError,
) -> String {
    error.public_code().to_string()
}

fn apply_progress(store: &mut DownloadStore, id: &str, update: ProgressUpdate) {
    if let Some(entry) = store.entries.get_mut(id) {
        entry.state.phase = update.phase;
        entry.state.downloaded = update.downloaded;
        entry.state.total = update.total;
        entry.state.percent = update.percent.min(100);
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

pub(super) fn list_locked(store: &DownloadStore) -> Vec<ModelDownloadState> {
    store
        .order
        .iter()
        .filter_map(|id| store.entries.get(id).map(|entry| entry.state.clone()))
        .collect()
}
