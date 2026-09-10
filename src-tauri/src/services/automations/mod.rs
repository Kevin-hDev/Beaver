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
mod runtime_lifecycle;
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

pub(crate) async fn read_runtime() -> Result<AutomationRuntime, AutomationError> {
    read_runtime_at(&crate::services::paths::data_dir()).await
}

pub(crate) use history_store::{all_at as all_history_at, append_at as append_history_at};
#[cfg(test)]
pub(crate) use runtime_lifecycle::{admit_at as admit_runtime_at, RuntimeAdmission};
pub(crate) use runtime_lifecycle::{
    mark_running_at as mark_runtime_running_at, mark_terminal_at as mark_runtime_terminal_at,
    remove_terminal_at as remove_runtime_terminal_at, runtime_at as read_runtime_at,
    terminal_at as read_terminal_at,
};
#[allow(unused_imports)]
pub use runtime_store::{
    recover_startup, reserved_automation_ids, scan_and_advance, AutomationOccurrence,
    AutomationRuntime, OccurrenceResult, OccurrenceResultStatus, OccurrenceState,
};
#[allow(unused_imports)]
pub use service::{
    create, delete, disable_missing_target, get, history, list, record_completion, update,
};
pub(crate) use service_mutations::record_completion_at;
#[allow(unused_imports)]
pub use store::{mutate, read_all};
pub use types::*;

pub(crate) const fn history_max_lines() -> usize {
    history_store::MAX_LINES
}

pub(crate) const fn history_max_line_bytes() -> usize {
    history_store::MAX_LINE_BYTES
}

#[cfg(test)]
mod audit_store_tests;
#[cfg(test)]
mod history_store_tests;
#[cfg(test)]
mod service_tests;

#[cfg(test)]
pub(crate) use runtime_store::recover_startup_at;
#[cfg(test)]
use service::{create_at, delete_at, get_at, list_at, update_at};
#[cfg(test)]
pub(crate) use store::{mutate_at as mutate_automations_at, read_all_at as read_automations_at};

#[cfg(test)]
mod migration_tests;
#[cfg(test)]
mod runtime_store_tests;
#[cfg(test)]
mod store_tests;
