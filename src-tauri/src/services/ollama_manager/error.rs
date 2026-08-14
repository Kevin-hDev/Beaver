// Les codes sont le contrat IPC complet; certains ne sont consommés qu'aux tâches suivantes.
#![allow(dead_code)]

use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(test, ts(rename_all = "kebab-case"))]
#[allow(clippy::enum_variant_names)]
pub enum OllamaErrorCode {
    OllamaUpdateCleanupPending,
    OllamaUpdateRecoveryRequired,
    OllamaRecoveryDeferred,
    OllamaModelStoreConflict,
    OllamaStorageUnavailable,
    OllamaJournalInvalid,
    OllamaBundleMissing,
    OllamaBundleInvalid,
    OllamaDownloadFailed,
    OllamaChecksumMismatch,
    OllamaExtractionFailed,
    OllamaValidationDeferred,
    OllamaOperationInProgress,
    OllamaOperationCancelled,
    OllamaClosing,
    OllamaStartFailed,
    OllamaStopFailed,
    OllamaSetupTimeout,
    OllamaUnavailable,
    OllamaInternal,
}

impl OllamaErrorCode {
    #[allow(dead_code)]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OllamaUpdateCleanupPending => "ollama-update-cleanup-pending",
            Self::OllamaUpdateRecoveryRequired => "ollama-update-recovery-required",
            Self::OllamaRecoveryDeferred => "ollama-recovery-deferred",
            Self::OllamaModelStoreConflict => "ollama-model-store-conflict",
            Self::OllamaStorageUnavailable => "ollama-storage-unavailable",
            Self::OllamaJournalInvalid => "ollama-journal-invalid",
            Self::OllamaBundleMissing => "ollama-bundle-missing",
            Self::OllamaBundleInvalid => "ollama-bundle-invalid",
            Self::OllamaDownloadFailed => "ollama-download-failed",
            Self::OllamaChecksumMismatch => "ollama-checksum-mismatch",
            Self::OllamaExtractionFailed => "ollama-extraction-failed",
            Self::OllamaValidationDeferred => "ollama-validation-deferred",
            Self::OllamaOperationInProgress => "ollama-operation-in-progress",
            Self::OllamaOperationCancelled => "ollama-operation-cancelled",
            Self::OllamaClosing => "ollama-closing",
            Self::OllamaStartFailed => "ollama-start-failed",
            Self::OllamaStopFailed => "ollama-stop-failed",
            Self::OllamaSetupTimeout => "ollama-setup-timeout",
            Self::OllamaUnavailable => "ollama-unavailable",
            Self::OllamaInternal => "ollama-internal",
        }
    }
}
