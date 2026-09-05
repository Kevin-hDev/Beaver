use super::super::{types::ExtensionRecord, OperationFailure};
use super::{InstallJobStore, InstallRequest, InstallStatus};

impl InstallJobStore {
    pub(crate) async fn wait_install(
        &self,
        request: InstallRequest,
    ) -> Result<ExtensionRecord, OperationFailure> {
        let job = self
            .start_reconciled(request)
            .await
            .map_err(|_| OperationFailure::InstallFailed)?;
        loop {
            let notify = self.notify.notified();
            let snapshot = self
                .snapshot()
                .map_err(|_| OperationFailure::InstallFailed)?;
            let view = snapshot
                .jobs
                .iter()
                .find(|current| current.id == job.id)
                .ok_or(OperationFailure::InstallFailed)?;
            if view.status == InstallStatus::Completed {
                return super::super::registry::find(
                    view.extension_id
                        .as_deref()
                        .ok_or(OperationFailure::InstallFailed)?,
                )
                .map_err(|_| OperationFailure::InstallFailed);
            }
            if view.status.terminal() {
                return Err(OperationFailure::InstallFailed);
            }
            tokio::select! { _ = notify => {}, _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {} }
        }
    }
}
