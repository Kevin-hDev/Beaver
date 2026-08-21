use std::future::Future;

use crate::services::owned_process::{OwnedProcess, OwnedProcessIdentity};
use crate::services::work_registry::ServiceWorkCancellation;

use super::process_receipt::SearxngProcessReceiptStore;

pub(super) async fn stabilize(
    pid: u32,
    deadline: tokio::time::Instant,
    cancel: &ServiceWorkCancellation,
) -> Result<OwnedProcessIdentity, String> {
    stabilize_with(
        super::process_receipt::store(),
        pid,
        cancel,
        OwnedProcess::identity,
        move |pid, cancel| async move {
            super::process::stable_identity(pid, deadline, &cancel).await
        },
    )
    .await
}

async fn stabilize_with<Initial, Stable, StableFuture>(
    store: SearxngProcessReceiptStore,
    pid: u32,
    cancel: &ServiceWorkCancellation,
    initial: Initial,
    stable: Stable,
) -> Result<OwnedProcessIdentity, String>
where
    Initial:
        FnOnce(
            u32,
        )
            -> Result<OwnedProcessIdentity, crate::services::owned_process::OwnedProcessError>,
    Stable: FnOnce(u32, ServiceWorkCancellation) -> StableFuture,
    StableFuture: Future<Output = Result<OwnedProcessIdentity, String>>,
{
    let initial_identity =
        initial(pid).map_err(|_| super::error_codes::START_FAILED.to_string())?;
    let pending_store = store.clone();
    super::startup::run_blocking(move || {
        pending_store
            .write_pending(&initial_identity)
            .map_err(|_| super::error_codes::START_FAILED.to_string())
    })
    .await?;

    let identity = stable(pid, cancel.clone()).await?;
    super::startup::run_blocking(move || {
        store
            .write(&identity)
            .map_err(|_| super::error_codes::START_FAILED.to_string())
    })
    .await?;
    Ok(identity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_exit::AppExitCoordinator;
    use crate::services::work_registry::ServiceWorkSupervisor;

    fn identity(executable: u128) -> OwnedProcessIdentity {
        OwnedProcessIdentity {
            pid: 42,
            native_start_time: 7,
            native_scope: 42,
            executable: (executable << 64) | (executable + 1),
        }
    }

    #[tokio::test]
    async fn pending_receipt_exists_before_stabilization_and_is_then_completed() {
        let root = tempfile::tempdir().unwrap();
        let store = SearxngProcessReceiptStore::at(root.path().join("receipt"));
        let observed = store.clone();
        let coordinator = AppExitCoordinator::initialize().unwrap();
        let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
        let admission = supervisor.try_admit().unwrap();

        let result = stabilize_with(
            store.clone(),
            42,
            &admission.cancellation(),
            |_| Ok(identity(11)),
            move |_, _| async move {
                assert_eq!(
                    observed.read().unwrap(),
                    super::super::process_receipt::SearxngProcessReceipt::pending(identity(11))
                );
                Ok(identity(13))
            },
        )
        .await
        .unwrap();

        assert_eq!(result, identity(13));
        assert_eq!(
            store.read().unwrap(),
            super::super::process_receipt::SearxngProcessReceipt::from_identity(identity(13))
        );
    }
}
