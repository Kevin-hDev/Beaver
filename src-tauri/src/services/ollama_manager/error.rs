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
    pub const ALL: [Self; 20] = [
        Self::OllamaUpdateCleanupPending,
        Self::OllamaUpdateRecoveryRequired,
        Self::OllamaRecoveryDeferred,
        Self::OllamaModelStoreConflict,
        Self::OllamaStorageUnavailable,
        Self::OllamaJournalInvalid,
        Self::OllamaBundleMissing,
        Self::OllamaBundleInvalid,
        Self::OllamaDownloadFailed,
        Self::OllamaChecksumMismatch,
        Self::OllamaExtractionFailed,
        Self::OllamaValidationDeferred,
        Self::OllamaOperationInProgress,
        Self::OllamaOperationCancelled,
        Self::OllamaClosing,
        Self::OllamaStartFailed,
        Self::OllamaStopFailed,
        Self::OllamaSetupTimeout,
        Self::OllamaUnavailable,
        Self::OllamaInternal,
    ];

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

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::OllamaUpdateCleanupPending => "ollama.errors.updateCleanupPending",
            Self::OllamaUpdateRecoveryRequired => "ollama.errors.updateRecoveryRequired",
            Self::OllamaRecoveryDeferred => "ollama.errors.recoveryDeferred",
            Self::OllamaModelStoreConflict => "ollama.errors.modelStoreConflict",
            Self::OllamaStorageUnavailable => "ollama.errors.storageUnavailable",
            Self::OllamaJournalInvalid => "ollama.errors.journalInvalid",
            Self::OllamaBundleMissing => "ollama.errors.bundleMissing",
            Self::OllamaBundleInvalid => "ollama.errors.bundleInvalid",
            Self::OllamaDownloadFailed => "ollama.errors.downloadFailed",
            Self::OllamaChecksumMismatch => "ollama.errors.checksumMismatch",
            Self::OllamaExtractionFailed => "ollama.errors.extractionFailed",
            Self::OllamaValidationDeferred => "ollama.errors.validationDeferred",
            Self::OllamaOperationInProgress => "ollama.errors.operationInProgress",
            Self::OllamaOperationCancelled => "ollama.errors.operationCancelled",
            Self::OllamaClosing => "ollama.errors.closing",
            Self::OllamaStartFailed => "ollama.errors.startFailed",
            Self::OllamaStopFailed => "ollama.errors.stopFailed",
            Self::OllamaSetupTimeout => "ollama.errors.setupTimeout",
            Self::OllamaUnavailable => "ollama.errors.unavailable",
            Self::OllamaInternal => "ollama.errors.internal",
        }
    }
}
