use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, ts(rename_all = "kebab-case"))]
pub enum UpdateOperationKind {
    AppRelease,
    OllamaBinary,
    OllamaModel,
    ForecastModel,
    VoiceModel,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, ts(rename_all = "kebab-case"))]
pub enum UpdateOperationStatus {
    Queued,
    Running,
    Cancelling,
    Completed,
    Failed,
    Cancelled,
}

impl UpdateOperationStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, ts(rename_all = "kebab-case"))]
pub enum UpdateOperationPhase {
    Waiting,
    Preparing,
    Downloading,
    Verifying,
    Extracting,
    Installing,
    Restarting,
    Recovering,
    RollingBack,
    Cleaning,
    Completed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, ts(rename_all = "kebab-case"))]
pub enum UpdateProgressMode {
    None,
    Indeterminate,
    Determinate,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct UpdateOperationSnapshot {
    pub id: String,
    #[cfg_attr(test, ts(type = "number"))]
    pub sequence: u64,
    pub kind: UpdateOperationKind,
    pub label: String,
    pub status: UpdateOperationStatus,
    pub phase: UpdateOperationPhase,
    pub progress_mode: UpdateProgressMode,
    pub percent: Option<u8>,
    pub queue_position: Option<u8>,
    pub can_cancel: bool,
    pub can_retry: bool,
    pub is_update: Option<bool>,
    pub error_key: Option<String>,
    #[cfg_attr(test, ts(type = "number | null"))]
    pub missing_bytes: Option<u64>,
}
