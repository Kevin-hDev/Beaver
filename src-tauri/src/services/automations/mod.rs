pub(crate) mod migration;
// Temporaire : la tâche 7 expose ces décisions via les commandes de migration.
mod audit_store;
mod history_cursor;
mod history_metadata;
mod history_store;
#[cfg(test)]
mod history_store_test_support;
#[allow(dead_code)]
pub(crate) mod migration_conflict;
mod migration_convert;
mod migration_files;
// Temporaire : les tâches 3 et 4 branchent ces stockages sur le service et le scheduler.
#[allow(dead_code)]
mod runtime_recovery;
#[allow(dead_code)]
mod runtime_scan;
#[allow(dead_code)]
mod runtime_store;
#[allow(dead_code)]
mod runtime_validation;
#[allow(dead_code)]
mod runtime_wire;
mod service;
mod service_helpers;
mod service_mutations;
#[allow(dead_code)]
mod store;
#[allow(dead_code)]
mod store_wire;
mod types;
mod validation;

use std::sync::OnceLock;
use tokio::sync::{Mutex, MutexGuard};

// ponytail: un verrou global suffit pour 64 automatisations ; segmenter si la contention est mesurée.
static STORE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub(crate) async fn store_lock() -> MutexGuard<'static, ()> {
    STORE_LOCK.get_or_init(|| Mutex::new(())).lock().await
}

#[cfg(test)]
pub(crate) use history_store::parse as parse_history;
pub(crate) use history_store::{all_at as all_history_at, append_at as append_history_at};
#[allow(unused_imports)]
pub use runtime_store::{
    recover_startup, reserved_automation_ids, scan_and_advance, AutomationOccurrence,
    AutomationRuntime, OccurrenceResult, OccurrenceResultStatus, OccurrenceState,
};
#[allow(unused_imports)]
pub use service::{
    create, delete, disable_missing_target, get, history, list, record_completion, update,
};
#[allow(unused_imports)]
pub use store::{mutate, read_all};
pub use types::*;

pub(crate) const fn history_max_lines() -> usize {
    history_store::MAX_LINES
}

#[cfg(test)]
pub(crate) use history_store_test_support::{
    append_with_atomic_writer as append_history_at_with_atomic_writer,
    append_with_read_observer as append_history_at_with_read_observer,
};

#[cfg(test)]
mod audit_store_tests;
#[cfg(test)]
mod history_store_tests;
#[cfg(test)]
mod service_tests;

#[cfg(test)]
use service::{create_at, delete_at, get_at, list_at, record_completion_at, update_at};

#[cfg(test)]
mod migration_tests;
#[cfg(test)]
mod runtime_store_tests;
#[cfg(test)]
mod store_tests;
