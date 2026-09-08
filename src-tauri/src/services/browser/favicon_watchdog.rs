use super::{favicon_policy::STALL_DIAGNOSTIC_DELAY, favicon_types::FaviconJob};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

// One timer per actual permit (at most four), owned and aborted with that permit.
// It never owns the CEF callback or returns a native permit on elapsed time.
pub(super) struct DownloadWatchdog {
    task: tauri::async_runtime::JoinHandle<()>,
    stalled: Arc<AtomicBool>,
    request: u64,
}
impl DownloadWatchdog {
    pub(super) fn start(job: &FaviconJob) -> Self {
        let stalled = Arc::new(AtomicBool::new(false));
        let detected = stalled.clone();
        let request = job.ticket.request;
        let task = tauri::async_runtime::spawn(detect_stall(detected, request));
        Self {
            task,
            stalled,
            request,
        }
    }
}
impl Drop for DownloadWatchdog {
    fn drop(&mut self) {
        self.task.abort();
        if self.stalled.load(Ordering::Acquire) {
            log::info!(
                "[browser] favicon native permit released request={}",
                self.request
            );
        }
    }
}

async fn detect_stall(detected: Arc<AtomicBool>, request: u64) {
    tokio::time::sleep(STALL_DIAGNOSTIC_DELAY).await;
    detected.store(true, Ordering::Release);
    log::warn!("[browser] favicon download stalled request={request} permit=retained recovery=native_callback_or_app_restart");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test(start_paused = true)]
    async fn diagnostic_fires_at_deadline_and_can_be_cancelled() {
        let detected = Arc::new(AtomicBool::new(false));
        let task = tokio::spawn(detect_stall(detected.clone(), 1));
        tokio::task::yield_now().await;
        assert!(!detected.load(Ordering::Acquire));
        tokio::time::advance(STALL_DIAGNOSTIC_DELAY).await;
        task.await.unwrap();
        assert!(detected.load(Ordering::Acquire));
        let cancelled = Arc::new(AtomicBool::new(false));
        let task = tokio::spawn(detect_stall(cancelled.clone(), 2));
        tokio::task::yield_now().await;
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        tokio::time::advance(STALL_DIAGNOSTIC_DELAY).await;
        assert!(!cancelled.load(Ordering::Acquire));
    }
}
