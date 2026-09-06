//! Shared cancellation boundary for existing installers and their job owner.
use super::install_jobs::InstallPhase;

pub(super) trait InstallSignal: Clone + Send + Sync + 'static {
    fn is_cancelled(&self) -> bool;
    fn producer_should_stop(&self) -> bool {
        self.is_cancelled()
    }
    fn after_producer_stopped(&self) -> Result<bool, super::process_runner::ProcessFailure> {
        if self.is_cancelled() {
            Err(super::process_runner::ProcessFailure::Interrupted)
        } else {
            Ok(false)
        }
    }
    fn storage_budget(&self) -> Result<u64, super::OperationFailure> {
        Ok(super::managed_tree::MAX_TOTAL_BYTES)
    }
    fn downloaded(&self, bytes: u64) -> bool {
        self.storage_budget().is_ok_and(|budget| bytes <= budget) && !self.producer_should_stop()
    }
    fn allows_volume(&self, bytes: u64) -> bool {
        self.storage_budget().is_ok_and(|budget| bytes <= budget)
    }
    fn resolved_git(
        &self,
        _source: &super::source_validation::GitSource,
    ) -> Result<(), super::OperationFailure> {
        Ok(())
    }
    fn lock_dependencies(&self, _root: &std::path::Path) -> Result<(), super::OperationFailure> {
        Ok(())
    }
    fn validate_replay(&self) -> Result<(), super::process_runner::ProcessFailure> {
        Ok(())
    }
    fn process_started(
        &self,
        _identity: crate::services::owned_process::OwnedProcessIdentity,
    ) -> Result<(), ()> {
        Ok(())
    }
    fn process_stopped(&self) -> Result<(), ()> {
        Ok(())
    }
    fn phase(&self, _phase: InstallPhase) -> Result<(), super::OperationFailure> {
        if self.is_cancelled() {
            Err(super::OperationFailure::InstallFailed)
        } else {
            Ok(())
        }
    }
}
impl InstallSignal for crate::services::work_registry::ServiceWorkCancellation {
    fn is_cancelled(&self) -> bool {
        self.is_cancelled()
    }
}
impl InstallSignal for super::install_jobs::InstallControl {
    fn is_cancelled(&self) -> bool {
        self.is_cancelled()
    }
    fn producer_should_stop(&self) -> bool {
        self.producer_should_stop()
    }
    fn after_producer_stopped(&self) -> Result<bool, super::process_runner::ProcessFailure> {
        self.after_producer_stopped()
            .map_err(|_| super::process_runner::ProcessFailure::Interrupted)
    }
    fn storage_budget(&self) -> Result<u64, super::OperationFailure> {
        self.storage_budget()
            .map_err(|_| super::OperationFailure::InstallFailed)
    }
    fn downloaded(&self, bytes: u64) -> bool {
        self.downloaded(bytes)
    }
    fn allows_volume(&self, bytes: u64) -> bool {
        self.allows_volume(bytes)
    }
    fn resolved_git(
        &self,
        source: &super::source_validation::GitSource,
    ) -> Result<(), super::OperationFailure> {
        self.resolved_git(source)
            .map_err(|_| super::OperationFailure::InstallFailed)
    }
    fn lock_dependencies(&self, root: &std::path::Path) -> Result<(), super::OperationFailure> {
        self.lock_dependencies(root)
            .map_err(|_| super::OperationFailure::ManifestInvalid)
    }
    fn validate_replay(&self) -> Result<(), super::process_runner::ProcessFailure> {
        self.validate_replay()
            .map_err(|_| super::process_runner::ProcessFailure::Failed)
    }
    fn process_started(
        &self,
        identity: crate::services::owned_process::OwnedProcessIdentity,
    ) -> Result<(), ()> {
        let mut checkpoint = self.saved().map_err(|_| ())?.ok_or(())?;
        checkpoint.native_process = Some(identity);
        self.save(checkpoint).map_err(|_| ())
    }
    fn process_stopped(&self) -> Result<(), ()> {
        let mut checkpoint = self.saved().map_err(|_| ())?.ok_or(())?;
        checkpoint.native_process = None;
        self.save(checkpoint).map_err(|_| ())
    }
    fn phase(&self, phase: InstallPhase) -> Result<(), super::OperationFailure> {
        self.checkpoint(phase)
            .map_err(|_| super::OperationFailure::InstallFailed)
    }
}
