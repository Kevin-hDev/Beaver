mod blocking;
mod constants;
mod durable_fs;
mod error;
mod fingerprint;
mod journal;
mod journal_store;
mod manager;
mod types;

#[cfg(test)]
mod blocking_tests;
#[cfg(test)]
mod document_tests;
#[cfg(test)]
mod durable_fs_test_support;
#[cfg(test)]
mod durable_fs_tests;
#[cfg(test)]
mod manager_tests;
#[cfg(all(test, windows))]
mod windows_durable_fs_tests;

#[allow(unused_imports)]
pub use error::OllamaErrorCode;
#[allow(unused_imports)]
pub use fingerprint::{BundleFingerprint, FingerprintError, OllamaVersion, Sha256Digest};
#[allow(unused_imports)]
pub use journal::{
    classify_migration_marker, DocumentError, OllamaJournalState, OllamaMigrationMarker,
    OllamaMigrationMarkerClassification, OllamaTransactionJournal,
};
pub use manager::OllamaManager;
#[allow(unused_imports)]
pub use types::{
    BundleState, DaemonState, OllamaEndpoint, OllamaProgressStage, OllamaRuntimeStatus,
    OllamaStartOutcome, OperationState,
};
