//! Shared cancellation boundary for existing installers and their job owner.
use super::install_jobs::InstallPhase;

pub(super) trait InstallSignal: Clone + Send + Sync + 'static {
    fn is_cancelled(&self) -> bool;
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
