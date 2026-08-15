// Les variantes décrivent le contrat IPC complet avant l'adoption des consommateurs.
#![allow(dead_code)]

use super::error::OllamaErrorCode;
use serde::Serialize;
use std::num::NonZeroU16;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum BundleState {
    Absent,
    Ready,
    TransactionPending,
    RecoveryRequired,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum DaemonState {
    Unavailable,
    Owned { endpoint: OllamaEndpoint },
    External { endpoint: OllamaEndpoint },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum OperationState {
    Idle,
    Installing,
    Updating,
    Recovering,
    Cancelling,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum OllamaProgressStage {
    Preparing,
    Downloading,
    Verifying,
    Extracting,
    Validating,
    Committing,
    Starting,
    Recovering,
    RollingBack,
    Cleaning,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum OllamaStartOutcome {
    OwnedStarted { endpoint: OllamaEndpoint },
    OwnedAlreadyRunning { endpoint: OllamaEndpoint },
    ExternalAvailable { endpoint: OllamaEndpoint },
    RejectedDuringShutdown,
    BlockedByRecovery { code: OllamaErrorCode },
    Failed { code: OllamaErrorCode },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub enum CancelOutcome {
    Cancelled,
    AlreadyIdle,
    RejectedDuringShutdown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OllamaCliArgs {
    Version,
    Create {
        model: String,
        modelfile: std::path::PathBuf,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OllamaCliOutput {
    pub success: bool,
}

impl OllamaCliArgs {
    pub(crate) fn validate(&self) -> Result<(), OllamaErrorCode> {
        match self {
            Self::Version => Ok(()),
            Self::Create { model, modelfile } => {
                if model.is_empty()
                    || model.len() > 128
                    || !model
                        .chars()
                        .all(|ch| ch.is_ascii_alphanumeric() || ":._-/".contains(ch))
                    || !modelfile.is_absolute()
                    || modelfile
                        .components()
                        .any(|component| matches!(component, std::path::Component::ParentDir))
                {
                    return Err(OllamaErrorCode::OllamaUnavailable);
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub struct OllamaEndpoint {
    port: NonZeroU16,
}

impl OllamaEndpoint {
    pub fn loopback(port: NonZeroU16) -> Self {
        Self { port }
    }

    pub fn as_http_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub(crate) fn port(&self) -> u16 {
        self.port.get()
    }

    pub fn try_from_http_url(raw: &str) -> Result<Self, OllamaErrorCode> {
        let parsed = url::Url::parse(raw).map_err(|_| OllamaErrorCode::OllamaUnavailable)?;
        let authority = raw
            .split_once("://")
            .and_then(|(_, rest)| rest.split(['/', '?', '#']).next())
            .unwrap_or_default();
        if parsed.scheme() != "http"
            || parsed.host_str() != Some("127.0.0.1")
            || parsed.port().and_then(NonZeroU16::new).is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || authority.contains('@')
            || parsed.query().is_some()
            || parsed.fragment().is_some()
            || !matches!(parsed.path(), "" | "/")
        {
            return Err(OllamaErrorCode::OllamaUnavailable);
        }
        Ok(Self::loopback(
            parsed
                .port()
                .and_then(NonZeroU16::new)
                .ok_or(OllamaErrorCode::OllamaUnavailable)?,
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, ts(rename_all = "snake_case"))]
pub struct OllamaRuntimeStatus {
    pub bundle: BundleState,
    pub daemon: DaemonState,
    pub operation: OperationState,
    pub progress: Option<OllamaProgressStage>,
    pub last_error: Option<OllamaErrorCode>,
}

impl OllamaRuntimeStatus {
    pub(crate) fn initial() -> Self {
        Self {
            bundle: BundleState::Absent,
            daemon: DaemonState::Unavailable,
            operation: OperationState::Idle,
            progress: None,
            last_error: None,
        }
    }
}
