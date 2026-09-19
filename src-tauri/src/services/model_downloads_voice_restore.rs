use super::model_downloads::ModelDownloadManager;
use super::model_downloads_store::{DownloadEntry, DownloadStore};
use super::model_downloads_types::{
    download_percentage, ModelDownloadKind, ModelDownloadState, ModelDownloadStatus,
    MAX_PENDING_DOWNLOADS,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

impl ModelDownloadManager {
    pub fn restore_voice_checkpoints(
        &self,
        data_dir: &std::path::Path,
        resource_dir: &std::path::Path,
    ) {
        let Ok(catalog) = crate::services::voice::download::load_catalog(resource_dir) else {
            return;
        };
        let checkpoints = crate::services::voice::download::discover_checkpoints(data_dir);
        let mut store = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        for checkpoint in checkpoints {
            let resource_is_current = catalog
                .entries
                .iter()
                .any(|entry| checkpoint.matches_entry(entry));
            let request_is_asr = catalog.entries.iter().any(|entry| {
                entry.id == checkpoint.requested_model_id
                    && entry.role == crate::services::voice::download::VoiceModelRole::Asr
            });
            if !resource_is_current || !request_is_asr {
                continue;
            }
            if store.entries.len() >= MAX_PENDING_DOWNLOADS
                || has_voice_model(&store, &checkpoint.requested_model_id)
            {
                continue;
            }
            let id = Uuid::new_v4().to_string();
            let mut state = ModelDownloadState::new(
                ModelDownloadKind::Voice,
                checkpoint.requested_model_id,
                false,
                id.clone(),
                ModelDownloadStatus::Suspended,
            );
            state.downloaded = checkpoint.durable_bytes;
            state.total = checkpoint.expected_bytes;
            state.percent =
                download_percentage(checkpoint.durable_bytes, checkpoint.expected_bytes);
            store.order.push_back(id.clone());
            store.entries.insert(
                id,
                DownloadEntry {
                    state,
                    cancel: CancellationToken::new(),
                },
            );
        }
    }
}

fn has_voice_model(store: &DownloadStore, model_id: &str) -> bool {
    store.entries.values().any(|entry| {
        entry.state.kind == ModelDownloadKind::Voice && entry.state.model_id == model_id
    })
}
