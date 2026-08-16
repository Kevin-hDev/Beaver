use super::{OwnedProcessError, OwnedProcessIdentity, OwnedProcessInspection};

pub(in crate::services::owned_process) fn inspect_for_recovery_with<Start, Identity>(
    pid: u32,
    expected_start_time: u64,
    read_start_time: Start,
    read_identity: Identity,
) -> Result<OwnedProcessInspection, OwnedProcessError>
where
    Start: FnOnce(u32) -> Option<u64>,
    Identity: FnOnce(u32) -> Result<OwnedProcessIdentity, OwnedProcessError>,
{
    let observed = read_start_time(pid).ok_or(OwnedProcessError::Admission)?;
    if observed != expected_start_time {
        return Ok(OwnedProcessInspection::Unowned);
    }
    read_identity(pid).map(OwnedProcessInspection::Owned)
}
