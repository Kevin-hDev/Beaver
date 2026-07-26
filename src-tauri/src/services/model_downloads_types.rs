use serde::{Deserialize, Serialize};

pub const MAX_PENDING_DOWNLOADS: usize = 16;
pub const EVENT_NAME: &str = "model-downloads-changed";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelDownloadKind {
    Ollama,
    Forecast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelDownloadStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelDownloadPhase {
    Starting,
    Downloading,
    PreparingRuntime,
    Installing,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadState {
    pub id: String,
    pub kind: ModelDownloadKind,
    pub model_id: String,
    pub is_update: bool,
    pub status: ModelDownloadStatus,
    pub phase: ModelDownloadPhase,
    pub percent: u8,
    pub downloaded: u64,
    pub total: u64,
    pub error_key: Option<String>,
}

impl ModelDownloadState {
    pub fn new(
        kind: ModelDownloadKind,
        model_id: String,
        is_update: bool,
        id: String,
        status: ModelDownloadStatus,
    ) -> Self {
        Self {
            id,
            kind,
            model_id,
            is_update,
            status,
            phase: ModelDownloadPhase::Starting,
            percent: 0,
            downloaded: 0,
            total: 0,
            error_key: None,
        }
    }
}
