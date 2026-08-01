use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

const SCAN_TIMEOUT: Duration = Duration::from_secs(600);

pub async fn run_scan<F>(work: F) -> ToolResult
where
    F: FnOnce(Arc<AtomicBool>) -> ToolResult + Send + 'static,
{
    run_scan_with_timeout(SCAN_TIMEOUT, work).await
}

pub fn scan_cancelled(cancelled: &AtomicBool) -> bool {
    cancelled.load(Ordering::Relaxed)
}

async fn run_scan_with_timeout<F>(duration: Duration, work: F) -> ToolResult
where
    F: FnOnce(Arc<AtomicBool>) -> ToolResult + Send + 'static,
{
    let cancelled = Arc::new(AtomicBool::new(false));
    let mut handle = tokio::task::spawn_blocking({
        let cancelled = Arc::clone(&cancelled);
        move || work(cancelled)
    });

    tokio::select! {
        result = &mut handle => match result {
            Ok(result) => result,
            Err(_) => ToolResult::error(
                "Le moteur de recherche interne s'est interrompu.",
                "scan_worker_failed",
                ToolErrorCategory::Internal,
                true,
            ),
        },
        _ = tokio::time::sleep(duration) => {
            cancelled.store(true, Ordering::Relaxed);
            handle.abort();
            ToolResult::error(
                format!("Timeout après {}s", duration.as_secs()),
                "scan_timeout",
                ToolErrorCategory::Timeout,
                true,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{run_scan_with_timeout, scan_cancelled};
    use crate::services::agent_local::types_tools::ToolResult;
    use std::time::Duration;

    #[tokio::test]
    async fn returns_completed_scan_result() {
        let result = run_scan_with_timeout(Duration::from_secs(1), |_| ToolResult::ok("ok")).await;
        assert!(!result.is_error);
        assert_eq!(result.content, "ok");
    }

    #[tokio::test]
    async fn times_out_long_scan() {
        let result = run_scan_with_timeout(Duration::from_millis(10), |cancelled| {
            while !scan_cancelled(&cancelled) {
                std::thread::sleep(Duration::from_millis(5));
            }
            ToolResult::ok("cancelled")
        })
        .await;

        assert!(result.is_error);
        assert!(result.content.contains("Timeout après 0s"));
    }
}
