use super::{InstallControl, InstallInterruption, InstallProgress};

impl super::InstallJobStore {
    pub(super) fn available_disk_space(&self) -> Result<u64, String> {
        #[cfg(test)]
        if let Some(free) = &self.free_bytes_for_test {
            return Ok(free.load(std::sync::atomic::Ordering::SeqCst));
        }
        super::disk_policy::free_bytes(&crate::services::paths::data_dir())
    }
}

#[derive(Clone, Default)]
pub(super) struct DiskMonitor {
    pub sampling: std::sync::Arc<std::sync::Mutex<()>>,
    pub sampled_at: Option<std::time::Instant>,
    pub stop: Option<InstallInterruption>,
    pub downloaded: Option<u64>,
}

impl InstallControl {
    pub(in crate::services::extensions) fn storage_budget(
        &self,
    ) -> Result<u64, InstallInterruption> {
        self.saved()?
            .map(|checkpoint| checkpoint.allowance.approved_total_bytes)
            .ok_or(InstallInterruption::Failed)
    }
    pub(in crate::services::extensions) fn producer_should_stop(&self) -> bool {
        self.is_cancelled() || self.poll_disk(false).is_err()
    }
    pub(in crate::services::extensions) fn downloaded(&self, bytes: u64) -> bool {
        if !self.allows_volume(bytes) {
            return false;
        }
        let Ok(mut state) = self.store.lock() else {
            return false;
        };
        let Ok(index) = state.index(&self.id) else {
            return false;
        };
        let job = &mut state.jobs[index];
        job.monitor.downloaded = Some(bytes);
        drop(state);
        !self.producer_should_stop()
    }
    pub(in crate::services::extensions) fn allows_volume(&self, bytes: u64) -> bool {
        let Ok(mut state) = self.store.lock() else {
            return false;
        };
        let Ok(index) = state.index(&self.id) else {
            return false;
        };
        let job = &mut state.jobs[index];
        if let Some(checkpoint) = &job.checkpoint {
            if bytes > checkpoint.allowance.approved_total_bytes {
                job.monitor.stop = Some(if checkpoint.allowance.confirmation_used {
                    InstallInterruption::InsufficientSpace
                } else {
                    InstallInterruption::Confirmation
                });
            }
        }
        drop(state);
        !self.producer_should_stop()
    }
    pub(super) fn disk_interruption(&self) -> Option<InstallInterruption> {
        let state = self.store.lock().ok()?;
        state.jobs[state.index(&self.id).ok()?].monitor.stop.clone()
    }
    fn poll_disk(&self, force: bool) -> Result<(), InstallInterruption> {
        self.poll_disk_with(force, |checkpoint| {
            super::disk_usage::measure(checkpoint).and_then(|occupied| {
                self.store
                    .available_disk_space()
                    .map(|free| (occupied, free))
                    .map_err(|_| InstallInterruption::Failed)
            })
        })
    }
    pub(super) fn poll_disk_with(
        &self,
        force: bool,
        measure: impl FnOnce(
            &super::checkpoint::InstallCheckpoint,
        ) -> Result<(u64, u64), InstallInterruption>,
    ) -> Result<(), InstallInterruption> {
        let sampling = {
            let state = self.store.lock().map_err(|_| InstallInterruption::Failed)?;
            state.jobs[state
                .index(&self.id)
                .map_err(|_| InstallInterruption::Failed)?]
            .monitor
            .sampling
            .clone()
        };
        // Serialize scans without holding the state lock: cancel remains responsive.
        let _sampling = sampling.lock().map_err(|_| InstallInterruption::Failed)?;
        let (checkpoint, phase, downloaded) = {
            let mut state = self.store.lock().map_err(|_| InstallInterruption::Failed)?;
            let index = state
                .index(&self.id)
                .map_err(|_| InstallInterruption::Failed)?;
            let job = &mut state.jobs[index];
            if !force
                && job
                    .monitor
                    .sampled_at
                    .is_some_and(|at| at.elapsed() < self.store.disk_policy.poll_interval)
            {
                return job.monitor.stop.clone().map_or(Ok(()), Err);
            }
            let Some(checkpoint) = job.checkpoint.clone() else {
                return Ok(());
            };
            (checkpoint, job.view.phase, job.monitor.downloaded)
        };
        let sample = measure(&checkpoint);
        let result = match sample {
            Ok((occupied, free)) => self
                .progress(InstallProgress {
                    phase,
                    downloaded_bytes: downloaded,
                    download_total_bytes: None,
                    occupied_bytes: occupied,
                    free_bytes: Some(free),
                })
                .and_then(|()| {
                    checkpoint
                        .allowance
                        .check(occupied, free, self.store.disk_policy)
                }),
            Err(error) => Err(error),
        };
        let mut state = self.store.lock().map_err(|_| InstallInterruption::Failed)?;
        let index = state
            .index(&self.id)
            .map_err(|_| InstallInterruption::Failed)?;
        let monitor = &mut state.jobs[index].monitor;
        // Rest starts after traversal and publication, even when either fails.
        monitor.sampled_at = Some(std::time::Instant::now());
        // Preserve a stop requested concurrently by a transfer callback.
        let reason = match result {
            Err(InstallInterruption::Confirmation) | Ok(()) => {
                monitor.stop.clone().or_else(|| result.err())
            }
            Err(error) => Some(error),
        };
        monitor.stop = reason.clone();
        reason.map_or(Ok(()), Err)
    }
    /// The caller reaped its process or returned from libgit2 before this wait.
    pub(in crate::services::extensions) fn after_producer_stopped(
        &self,
    ) -> Result<bool, InstallInterruption> {
        if self.app_cancel.is_cancelled() {
            return Err(InstallInterruption::AppClosing);
        }
        if self.is_cancelled() {
            return Err(InstallInterruption::Cancelled);
        }
        if self
            .saved()?
            .is_some_and(|checkpoint| checkpoint.native_process.is_some())
        {
            return Err(InstallInterruption::Failed);
        }
        match self.poll_disk(true) {
            Ok(()) => Ok(false),
            Err(InstallInterruption::Confirmation) => {
                tokio::runtime::Handle::current().block_on(self.await_confirmation())?;
                self.poll_disk(true)?;
                Ok(true)
            }
            Err(error) => Err(error),
        }
    }
}
