mod blocking;
mod canonical_executable;
mod cleanup;
mod cleanup_inspection;
mod constants;
mod durable_fs;
mod error;
mod fingerprint;
mod journal;
mod journal_store;
mod manager;
mod migration;
mod path_identity;
mod path_identity_resolver;
mod port;
mod probe;
mod probe_http;
mod probe_ownership;
mod probe_runner;
mod probe_support;
#[allow(dead_code)]
mod process;
#[allow(dead_code)]
mod process_owned;
#[allow(dead_code)]
mod process_receipt;
#[allow(dead_code)]
mod process_receipt_recovery;
mod recovery;
mod recovery_decision;
mod recovery_decision_rules;
mod recovery_entry;
mod rollback;
mod spawn_environment;
#[cfg(unix)]
#[allow(dead_code)]
mod spawn_gate_unix;
#[cfg(windows)]
#[allow(dead_code)]
mod spawn_gate_windows;
mod spawn_profile;
mod spawn_profile_paths;
mod types;

#[cfg(test)]
mod blocking_tests;
#[cfg(test)]
mod cleanup_tests;
#[cfg(test)]
mod document_tests;
#[cfg(test)]
mod durable_fs_test_support;
#[cfg(test)]
mod durable_fs_tests;
#[cfg(test)]
mod manager_tests;
#[cfg(test)]
mod migration_tests;
#[cfg(test)]
mod port_tests;
#[cfg(test)]
mod probe_http_tests;
#[cfg(test)]
mod probe_tests;
#[cfg(test)]
mod process_receipt_tests;
#[cfg(test)]
mod process_tests;
#[cfg(test)]
mod recovery_decision_tests;
#[cfg(test)]
mod recovery_tests;
#[cfg(test)]
mod rollback_tests;
#[cfg(test)]
mod spawn_profile_attempt_tests;
#[cfg(test)]
mod spawn_profile_environment_tests;
#[cfg(test)]
mod spawn_profile_test_support;
#[cfg(test)]
mod spawn_profile_tests;
#[cfg(all(test, windows))]
mod windows_durable_fs_tests;
#[cfg(all(test, windows))]
mod windows_path_identity_tests;
#[cfg(all(test, windows))]
mod windows_spawn_gate_tests;

#[cfg(windows)]
pub(crate) use canonical_executable::windows_image_identity_from_path;
#[allow(unused_imports)]
pub(crate) use canonical_executable::{CanonicalExecutable, NativeFileIdentity};
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
pub(crate) use path_identity::{
    CanonicalDirectory, NativeDirectoryIdentity, PathIdentityResolver, ValidatedPathComponent,
    VerifiedDirectoryLocation,
};
#[allow(unused_imports)]
pub(crate) use path_identity_resolver::NativePathIdentityResolver;
#[allow(unused_imports)]
pub use port::{DefaultOllamaPortAllocator, OllamaPortAllocator};
#[allow(unused_imports)]
pub use probe::{OllamaTargetProbe, OwnedOllamaTargetProbe, PreparedBundle, TargetValidation};
#[allow(unused_imports)]
pub use recovery::{RecoveryOutcome, RecoveryReason};
#[allow(unused_imports)]
pub(crate) use spawn_profile::{FrozenEnvironment, OllamaSpawnAttempt, OllamaSpawnProfile};
#[allow(unused_imports)]
pub use types::{
    BundleState, DaemonState, OllamaEndpoint, OllamaProgressStage, OllamaRuntimeStatus,
    OllamaStartOutcome, OperationState,
};
