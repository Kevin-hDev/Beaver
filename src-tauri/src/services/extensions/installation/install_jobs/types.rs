use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum InstallRequest {
    Local {
        path: String,
    },
    Git {
        locator: String,
    },
    Npm {
        locator: String,
    },
    Update {
        #[serde(rename = "extensionId")]
        extension_id: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub(crate) enum InstallKind {
    Local,
    Git,
    Npm,
    Update,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub(crate) enum InstallStatus {
    Queued,
    Running,
    AwaitingConfirmation,
    Cancelling,
    Completed,
    Cancelled,
    Failed,
    Interrupted,
}
impl InstallStatus {
    pub(super) fn terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Cancelled | Self::Failed | Self::Interrupted
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub(crate) enum InstallPhase {
    Resolving,
    Downloading,
    Dependencies,
    Validating,
    BuildingUi,
    Publishing,
    Cleaning,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum QueueBlocker {
    Confirmation {
        #[serde(rename = "jobId")]
        job_id: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstallJobView {
    pub id: String,
    #[cfg_attr(test, ts(type = "number"))]
    pub revision: u64,
    pub kind: InstallKind,
    pub display_name: String,
    pub status: InstallStatus,
    pub phase: InstallPhase,
    #[cfg_attr(test, ts(type = "number | null"))]
    pub downloaded_bytes: Option<u64>,
    #[cfg_attr(test, ts(type = "number | null"))]
    pub download_total_bytes: Option<u64>,
    #[cfg_attr(test, ts(type = "number"))]
    pub occupied_bytes: u64,
    #[cfg_attr(test, ts(type = "number | null"))]
    pub free_bytes: Option<u64>,
    pub confirmation_id: Option<String>,
    pub error_code: Option<String>,
    pub extension_id: Option<String>,
    pub can_cancel: bool,
    pub can_resume: bool,
    pub queue_blocker: Option<QueueBlocker>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub(crate) struct InstallJobsSnapshot {
    #[cfg_attr(test, ts(type = "number"))]
    pub revision: u64,
    pub jobs: Vec<InstallJobView>,
}
