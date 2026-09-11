pub(crate) mod actor_context;
mod audit_store;
mod history_cursor;
mod history_metadata;
mod history_store;
#[cfg(test)]
mod history_store_test_support;
pub(crate) mod migration;
pub(crate) mod migration_conflict;
mod migration_convert;
mod migration_files;
pub(crate) mod next_fire;
#[cfg(test)]
mod next_fire_tests;
mod runtime_lifecycle;
mod runtime_recovery;
mod runtime_retired;
mod runtime_scan;
mod runtime_store;
mod runtime_validation;
mod runtime_wire;
mod service;
mod service_helpers;
mod service_mutations;
mod store;
mod store_wire;
mod text_validation;
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

pub(crate) use runtime_scan::scan_definitions_at as scan_and_advance_at;

#[cfg(test)]
pub(crate) use history_store::all_at as all_history_at;
pub(crate) use history_store::append_at as append_history_at;
pub(crate) use runtime_lifecycle::{
    mark_running_at as mark_runtime_running_at, mark_terminal_at as mark_runtime_terminal_at,
    remove_terminal_unlocked_at, runtime_at as read_runtime_at, terminal_unlocked_at,
};
#[cfg(test)]
pub(crate) use runtime_retired::reserved_at as reserved_automation_ids_at;
pub(crate) use runtime_retired::{
    release_unlocked_at as release_retired_unlocked_at, reserved_unlocked_at,
    retire_if_referenced_unlocked_at,
};
pub use runtime_store::{
    recover_startup, AutomationOccurrence, AutomationRuntime, OccurrenceResult,
    OccurrenceResultStatus, OccurrenceState,
};
pub use service::{create, delete, disable_missing_target, get, history, list, update};
#[cfg(test)]
pub(crate) use service_mutations::record_completion_at;
pub(crate) use service_mutations::record_completion_unlocked_at;
#[cfg(test)]
pub(crate) use store::mutate;
pub use store::read_all;
pub(crate) use text_validation::{
    validate_multiline_text, validate_optional_multiline_text, validate_single_line_text,
};
pub use types::*;
pub(crate) use validation::validate_schedule;

pub(crate) const fn history_max_lines() -> usize {
    history_store::MAX_LINES
}

pub(crate) const fn history_max_line_bytes() -> usize {
    history_store::MAX_LINE_BYTES
}

#[cfg(test)]
mod actor_context_tests;
#[cfg(test)]
mod audit_store_tests;
#[cfg(test)]
mod automation_e2e_tests;
#[cfg(test)]
mod automation_race_tests;
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
