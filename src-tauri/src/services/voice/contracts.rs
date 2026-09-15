use super::types::{VoiceLanguage, VoicePhase};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum VoiceDestination {
    Draft { draft_key: String },
    Trial { trial_id: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub enum VoiceDeliveryOutcome {
    Inserted,
    AlreadyInserted,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum VoiceAction {
    Start {
        destination: VoiceDestination,
        #[cfg_attr(test, ts(type = "number"))]
        context_generation: u64,
        language: Option<VoiceLanguage>,
    },
    Validate {
        operation_id: String,
    },
    CancelInsertion {
        operation_id: String,
    },
    AbandonTrial {
        trial_id: String,
    },
    DeleteRecovery {
        recovery_id: String,
    },
    RestoreRecovery {
        recovery_id: String,
        draft_key: String,
    },
    AcknowledgeDelivery {
        result_id: String,
        outcome: VoiceDeliveryOutcome,
    },
    DestinationClosed {
        destination: VoiceDestination,
    },
    MessageAccepted {
        draft_key: String,
        send_id: String,
    },
    Install {
        model_id: String,
    },
    Resume {
        transfer_id: String,
    },
    CancelDownload {
        transfer_id: String,
    },
    Uninstall {
        model_id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct VoiceDevice {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct VoiceOperationSnapshot {
    pub id: String,
    pub destination: VoiceDestination,
    #[cfg_attr(test, ts(type = "number"))]
    pub context_generation: u64,
    #[cfg_attr(test, ts(type = "number"))]
    pub capture_ms: u64,
    #[cfg_attr(test, ts(type = "number"))]
    pub speech_ms: u64,
    pub capture_incomplete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct VoiceRecoverySnapshot {
    pub id: String,
    pub draft_key: Option<String>,
    #[cfg_attr(test, ts(type = "number"))]
    pub capture_ms: u64,
    pub status: VoiceRecoveryState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub enum VoiceRecoveryState {
    Preparing,
    Ready,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct VoiceDeliverySnapshot {
    pub id: String,
    pub draft_key: String,
    pub text: String,
}

impl Drop for VoiceDeliverySnapshot {
    fn drop(&mut self) {
        self.text.zeroize();
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct VoiceTrialResult {
    pub trial_id: String,
    pub text: String,
}

impl Drop for VoiceTrialResult {
    fn drop(&mut self) {
        self.text.zeroize();
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct VoiceSnapshot {
    #[cfg_attr(test, ts(type = "number"))]
    pub revision: u64,
    pub phase: VoicePhase,
    pub operation: Option<VoiceOperationSnapshot>,
    pub recovery: Option<VoiceRecoverySnapshot>,
    pub delivery: Option<VoiceDeliverySnapshot>,
    pub trial_result: Option<VoiceTrialResult>,
    pub error: Option<super::errors::VoiceError>,
}
