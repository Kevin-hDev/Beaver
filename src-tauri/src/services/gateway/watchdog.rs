use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

pub struct StallWatchdog {
    last_activity_ms: Arc<AtomicI64>,
    armed: Arc<AtomicBool>,
    cancel: CancellationToken,
}

impl StallWatchdog {
    pub fn spawn(timeout: Duration, on_stall: impl Fn(Duration) + Send + 'static) -> Self {
        let last_activity_ms = Arc::new(AtomicI64::new(now_ms()));
        let armed = Arc::new(AtomicBool::new(false));
        let cancel = CancellationToken::new();

        let la = last_activity_ms.clone();
        let ar = armed.clone();
        let ct = cancel.clone();
        let check_interval = Duration::from_secs(5).min(timeout / 3);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = ct.cancelled() => break,
                    _ = tokio::time::sleep(check_interval) => {}
                }
                if !ar.load(Ordering::Relaxed) {
                    continue;
                }
                let last = la.load(Ordering::Relaxed);
                let idle_ms = now_ms() - last;
                if idle_ms > timeout.as_millis() as i64 {
                    on_stall(Duration::from_millis(idle_ms as u64));
                }
            }
        });

        Self {
            last_activity_ms,
            armed,
            cancel,
        }
    }

    pub fn arm(&self) {
        self.touch();
        self.armed.store(true, Ordering::Relaxed);
    }

    #[cfg(test)]
    pub fn disarm(&self) {
        self.armed.store(false, Ordering::Relaxed);
    }

    pub fn touch(&self) {
        self.last_activity_ms.store(now_ms(), Ordering::Relaxed);
    }

    pub fn stop(&self) {
        self.cancel.cancel();
    }

    #[cfg(test)]
    pub fn is_armed(&self) -> bool {
        self.armed.load(Ordering::Relaxed)
    }
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[tokio::test]
    async fn arm_and_disarm() {
        let wd = StallWatchdog::spawn(Duration::from_secs(60), |_| {});
        assert!(!wd.is_armed());
        wd.arm();
        assert!(wd.is_armed());
        wd.disarm();
        assert!(!wd.is_armed());
        wd.stop();
    }

    #[tokio::test]
    async fn touch_updates_activity() {
        let wd = StallWatchdog::spawn(Duration::from_secs(60), |_| {});
        let before = wd.last_activity_ms.load(Ordering::Relaxed);
        tokio::time::sleep(Duration::from_millis(10)).await;
        wd.touch();
        let after = wd.last_activity_ms.load(Ordering::Relaxed);
        assert!(after > before);
        wd.stop();
    }

    #[tokio::test]
    async fn stall_triggers_callback() {
        let stall_count = Arc::new(AtomicU32::new(0));
        let sc = stall_count.clone();

        let wd = StallWatchdog::spawn(Duration::from_millis(50), move |_| {
            sc.fetch_add(1, Ordering::Relaxed);
        });
        wd.arm();
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(stall_count.load(Ordering::Relaxed) > 0);
        wd.stop();
    }

    #[tokio::test]
    async fn no_stall_when_disarmed() {
        let stall_count = Arc::new(AtomicU32::new(0));
        let sc = stall_count.clone();

        let wd = StallWatchdog::spawn(Duration::from_millis(50), move |_| {
            sc.fetch_add(1, Ordering::Relaxed);
        });
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(stall_count.load(Ordering::Relaxed), 0);
        wd.stop();
    }
}
