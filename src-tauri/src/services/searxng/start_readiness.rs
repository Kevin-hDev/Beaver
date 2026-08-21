use std::sync::atomic::Ordering;

use crate::services::work_registry::ServiceWorkCancellation;

use super::lifecycle::{shutdown_error, SearxngSidecar};

pub(super) fn ensure_start_active(
    sidecar: &SearxngSidecar,
    cancel: &ServiceWorkCancellation,
    generation: u64,
) -> Result<(), String> {
    if cancel.is_cancelled() || sidecar.publication_generation.load(Ordering::Acquire) != generation
    {
        return Err(shutdown_error());
    }
    Ok(())
}

pub(super) fn run_if_start_active<Cleanup>(
    sidecar: &SearxngSidecar,
    cancel: &ServiceWorkCancellation,
    generation: u64,
    cleanup: Cleanup,
) -> Result<(), String>
where
    Cleanup: FnOnce(),
{
    ensure_start_active(sidecar, cancel, generation)?;
    cleanup();
    Ok(())
}
