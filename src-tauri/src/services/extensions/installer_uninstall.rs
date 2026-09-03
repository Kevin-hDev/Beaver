use super::installer::{blocking, extension_runtime, is_managed};
use super::OperationFailure;
use crate::services::work_registry::ServiceWorkCancellation;

pub async fn uninstall(id: &str, deadline: std::time::Instant) -> Result<bool, OperationFailure> {
    let current = super::registry::find(id).map_err(|_| OperationFailure::UninstallFailed)?;
    let id = id.to_string();
    let runtime = extension_runtime()?;
    let identity = super::host_identity::HostIdentity::ThirdParty(id.clone());
    let work = runtime.work.clone();
    work.run_operation(move |cancel| async move {
        ensure_uninstall_active(&cancel)?;
        crate::services::agent_local::permission_gate::clear_extension(&id).await;
        let reminder = current.sensitive_access_granted;
        // La désactivation persiste avant l'arrêt afin qu'un arbre non confirmé
        // ne puisse jamais être remplacé ni redevenir actif au redémarrage.
        super::registry::set_enabled(&id, false, false)
            .await
            .map_err(|_| OperationFailure::UninstallFailed)?;
        let stopped = runtime.revoke_extension(&identity, deadline).await;
        super::host_stop_boundary::after_confirmed_stop(
            stopped,
            OperationFailure::HostUnavailable,
            async move {
                let persisted_reminder =
                    super::registry::remove(&id).map_err(|_| OperationFailure::UninstallFailed)?;
                super::ui_artifact_store::remove(&current)
                    .map_err(|_| OperationFailure::StorageFailed)?;
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
                result.map(|_| reminder || persisted_reminder)
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
