use crate::services::work_registry::ServiceWorkCancellation;

pub(super) async fn detect_owned(cancel: &ServiceWorkCancellation) -> Option<(u64, Option<u64>)> {
    if !cfg!(target_arch = "aarch64") || cancel.is_cancelled() {
        return None;
    }
    // macOS exposes unified memory, so the OS memory snapshot is the relevant
    // authority. It also avoids competing with CEF's process burst at startup.
    let snapshot = tokio::task::spawn_blocking(|| {
        let mut system = sysinfo::System::new();
        system.refresh_memory();
        let total_mb = system.total_memory() / 1_048_576;
        let used_mb = system.used_memory() / 1_048_576;
        (total_mb > 0).then_some((total_mb, Some(used_mb.min(total_mb))))
    })
    .await
    .ok()
    .flatten();
    (!cancel.is_cancelled()).then_some(snapshot).flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_exit::AppExitCoordinator;
    use crate::services::work_registry::ServiceWorkSupervisor;

    #[cfg(target_arch = "aarch64")]
    #[tokio::test]
    async fn apple_silicon_probe_reads_real_unified_memory() {
        let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
        let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
        let admission = supervisor.try_admit().expect("probe admission");
        // One real probe validates the macOS wiring. Retry behavior is tested with a
        // deterministic fake so this hardware test does not amplify system pressure.
        let (total_mb, used_mb) = detect_owned(&admission.cancellation())
            .await
            .expect("Apple unified memory");

        assert!(total_mb >= 4_096);
        assert!(used_mb.expect("unified usage") <= total_mb);
        drop(admission);
    }
}
