use super::installer::{blocking, extension_runtime, is_managed};
use super::OperationFailure;
use crate::services::work_registry::ServiceWorkCancellation;

pub async fn uninstall(id: &str) -> Result<(), OperationFailure> {
    let current = super::registry::find(id).map_err(|_| OperationFailure::UninstallFailed)?;
    let id = id.to_string();
    let runtime = extension_runtime()?;
    let identity = super::host_identity::HostIdentity::ThirdParty(id.clone());
    let work = runtime.work.clone();
    work.run_operation(move |cancel| async move {
        ensure_uninstall_active(&cancel)?;
        let stopped = tokio::select! {
            _ = cancel.cancelled() => return Err(OperationFailure::HostUnavailable),
            stopped = runtime.stop_channel(&identity, None) => stopped,
        };
        ensure_uninstall_active(&cancel)?;
        super::host_stop_boundary::after_confirmed_stop(
            stopped,
            OperationFailure::HostUnavailable,
            async move {
                if super::registry::remove(&id).is_err() {
                    return Err(OperationFailure::UninstallFailed);
                }
                let result = if is_managed(&current) {
                    let record = current.clone();
                    blocking(
                        move || {
                            super::managed_store::remove_record(&record)
                                .map_err(|_| OperationFailure::StorageFailed)
                        },
                        OperationFailure::UninstallFailed,
                    )
                    .await
                } else {
                    Ok(())
                };
                result
            },
        )
        .await
    })
    .await
    .map_err(|error| error.operation_failure())?
}

fn ensure_uninstall_active(cancel: &ServiceWorkCancellation) -> Result<(), OperationFailure> {
    if cancel.is_cancelled() {
        return Err(OperationFailure::HostUnavailable);
    }
    Ok(())
}

#[cfg(test)]
#[path = "installer_tests.rs"]
mod tests;
