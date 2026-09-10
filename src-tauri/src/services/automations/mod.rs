pub(crate) mod migration;
// Temporaire : la tâche 7 expose ces décisions via les commandes de migration.
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
#[allow(dead_code)]
mod store;
#[allow(dead_code)]
mod store_wire;

use std::sync::OnceLock;
use tokio::sync::{Mutex, MutexGuard};

// ponytail: un verrou global suffit pour 64 automatisations ; segmenter si la contention est mesurée.
static STORE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub(crate) async fn store_lock() -> MutexGuard<'static, ()> {
    STORE_LOCK.get_or_init(|| Mutex::new(())).lock().await
}

#[allow(unused_imports)]
pub use runtime_store::{
    recover_startup, reserved_automation_ids, scan_and_advance, AutomationOccurrence,
    AutomationRuntime, OccurrenceResult, OccurrenceResultStatus, OccurrenceState,
};
#[allow(unused_imports)]
pub use store::{mutate, read_all};

#[cfg(test)]
mod migration_tests;
#[cfg(test)]
mod runtime_store_tests;
#[cfg(test)]
mod store_tests;
