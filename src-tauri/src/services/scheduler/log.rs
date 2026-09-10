#[path = "log_store.rs"]
mod store;

use crate::models::WakeupRun;
use std::path::PathBuf;

fn log_path() -> PathBuf {
    crate::services::paths::data_dir()
        .join("logs")
        .join("wakeups.jsonl")
}

pub async fn list_runs(wakeup_id: Option<&str>) -> Result<Vec<WakeupRun>, String> {
    store::list_runs_at(&log_path(), wakeup_id).await
}

#[allow(dead_code)]
pub(crate) const fn max_lines() -> usize {
    crate::services::automations::history_max_lines()
}

#[cfg(test)]
#[path = "log_tests.rs"]
mod tests;
