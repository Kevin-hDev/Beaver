use super::lifecycle::SearxngSidecar;
use std::sync::atomic::Ordering;
use std::time::Instant;

impl SearxngSidecar {
    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.work.begin_closing();
        self.publication_generation.fetch_add(1, Ordering::AcqRel);
        let process_stopped = stop_published_process(self, deadline).await;
        self.work.stop_and_wait(deadline).await && process_stopped
    }

    #[cfg(test)]
    pub(crate) fn try_admit_start_for_test(&self) -> Result<(), ()> {
        let admission = self.work.try_admit_server().map_err(|_| ())?;
        drop(admission);
        Ok(())
    }
}

async fn stop_published_process(sidecar: &SearxngSidecar, deadline: Instant) -> bool {
    let Ok(mut process) = tokio::time::timeout_at(
        tokio::time::Instant::from_std(deadline),
        sidecar.process.lock(),
    )
    .await
    else {
        return false;
    };
    let handle = process.take();
    drop(process);
    let Some(handle) = handle else { return true };
    tokio::time::timeout_at(
        tokio::time::Instant::from_std(deadline),
        super::process::kill_child_process(handle.child),
    )
    .await
    .is_ok()
}
