mod archive_recovery;
mod blocking;
mod bundle_install;
mod bundle_receipt;
mod canonical_executable;
mod cleanup;
mod cleanup_inspection;
mod constants;
mod download;
mod durable_fs;
mod error;
mod extract;
mod extract_archive;
#[cfg(test)]
mod extract_fixture;
mod extract_root;
mod extract_zip_validate;
mod fingerprint;
mod install;
mod install_archives;
mod install_confinement;
mod install_facade;
#[cfg(test)]
mod install_phases;
#[cfg(test)]
mod install_test_support;
mod journal;
mod journal_store;
mod manager;
mod migration;
mod path_identity;
mod path_identity_resolver;
pub(crate) mod polling;
mod port;
mod probe;
mod probe_http;
mod probe_ownership;
mod probe_runner;
mod probe_support;
#[allow(dead_code)]
mod process;
mod process_error;
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
mod recovery_helpers;
mod recovery_probe;
mod recovery_types;
mod release_fetch;
mod release_redirect;
pub(crate) mod release_source;
mod retry;
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
mod staging_recovery;
mod startup;
mod startup_recovery;
mod types;
mod update;

mod adoption;
#[cfg(test)]
mod adoption_tests;
#[cfg(test)]
mod blocking_tests;
#[cfg(test)]
mod cleanup_tests;
#[cfg(test)]
mod document_tests;
#[cfg(test)]
mod download_tests;
#[cfg(test)]
mod durable_fs_test_support;
#[cfg(test)]
mod durable_fs_tests;
#[cfg(test)]
mod extract_tests;
#[cfg(test)]
mod historical_scenarios_tests;
#[cfg(test)]
mod install_tests;
#[cfg(test)]
mod layout_fixtures_tests;
#[cfg(test)]
mod manager_tests;
#[cfg(test)]
mod migration_tests;
#[cfg(test)]
mod polling_tests;
#[cfg(test)]
mod port_tests;
#[cfg(test)]
mod probe_http_tests;
#[cfg(test)]
mod probe_tests;
#[cfg(test)]
mod process_receipt_tests;
#[cfg(all(test, unix))]
mod process_tests;
#[cfg(test)]
mod recovery_decision_tests;
#[cfg(test)]
mod recovery_tests;
#[cfg(test)]
mod release_source_tests;
#[cfg(test)]
mod retry_tests;
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
#[cfg(test)]
mod startup_tests;
#[cfg(test)]
mod transaction_property_tests;
#[cfg(test)]
mod typescript_contract_tests;
#[cfg(test)]
mod update_completion_support;
#[cfg(test)]
mod update_completion_tests;
#[cfg(test)]
mod update_platform_tests;
#[cfg(test)]
mod update_tests;
#[cfg(all(test, windows))]
mod windows_durable_fs_tests;
#[cfg(all(test, windows))]
mod windows_path_identity_tests;
#[cfg(all(test, windows))]
mod windows_spawn_gate_tests;

#[allow(unused_imports)]
pub use bundle_receipt::{BundlePlatform, BundleReceipt};
#[cfg(windows)]
pub(crate) use canonical_executable::windows_image_identity_from_path;
#[allow(unused_imports)]
pub(crate) use canonical_executable::{CanonicalExecutable, NativeFileIdentity};
#[allow(unused_imports)]
pub use download::{download_archive, download_archives, verify_sha256};
#[allow(unused_imports)]
pub use error::OllamaErrorCode;
#[allow(unused_imports)]
pub use extract::{extract_archive, extract_archive_overlay};
#[allow(unused_imports)]
pub use fingerprint::{BundleFingerprint, FingerprintError, OllamaVersion, Sha256Digest};
#[allow(unused_imports)]
pub use install::{InstallOutcome, InstallRequest};
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
pub(crate) use process_owned::OwnedProcessSidecar;
#[allow(unused_imports)]
pub use recovery::{RecoveryOutcome, RecoveryReason};
#[allow(unused_imports)]
pub use release_source::{
    AllowlistedArchiveName, OllamaArchive, OllamaReleaseManifest, ValidatedHttpsUrl,
};
#[allow(unused_imports)]
pub(crate) use spawn_profile::{FrozenEnvironment, OllamaSpawnAttempt, OllamaSpawnProfile};
#[allow(unused_imports)]
pub(crate) use startup::{OllamaStartupBarrier, StartupBarrierState};
#[allow(unused_imports)]
pub use types::{
    BundleState, CancelOutcome, DaemonState, OllamaCliArgs, OllamaCliOutput, OllamaEndpoint,
    OllamaProgressStage, OllamaRuntimeStatus, OllamaStartOutcome, OperationState,
};
#[allow(unused_imports)]
pub use update::{UpdateOutcome, UpdateRequest, UpdateSidecar};
