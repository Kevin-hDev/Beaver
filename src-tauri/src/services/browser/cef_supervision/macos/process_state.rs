use super::super::CefUnavailableCategory;
use super::identity::MacProcessIdentity;
use super::process_syscalls::{signal_group_raw, MacSystemProbe};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MacProcessObservation {
    Alive,
    Stopped,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MacSignalObservation {
    Ready,
    Stopped,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MacSignalResult {
    Sent,
    Stopped,
}

pub(super) trait MacProcessActions: Send + Sync {
    fn observe(&self, identity: &MacProcessIdentity) -> MacProcessObservation;
    fn revalidate_before_signal(&self, identity: &MacProcessIdentity) -> MacSignalObservation;
    fn signal_group(
        &self,
        identity: &MacProcessIdentity,
    ) -> Result<MacSignalResult, CefUnavailableCategory>;
}

pub(super) struct MacSystemProcessActions;

impl MacProcessActions for MacSystemProcessActions {
    fn observe(&self, identity: &MacProcessIdentity) -> MacProcessObservation {
        observe_with(&MacSystemProbe, identity)
    }

    fn revalidate_before_signal(&self, identity: &MacProcessIdentity) -> MacSignalObservation {
        revalidate_with(&MacSystemProbe, identity)
    }

    fn signal_group(
        &self,
        identity: &MacProcessIdentity,
    ) -> Result<MacSignalResult, CefUnavailableCategory> {
        let group = i32::try_from(identity.process_group)
            .ok()
            .filter(|group| *group > 0)
            .ok_or(CefUnavailableCategory::Reaper)?;
        let (result, errno) = signal_group_raw(group);
        classify_signal_result(result, errno)
    }
}

pub(super) fn read_active_identity(pid: u32) -> Result<MacProcessIdentity, CefUnavailableCategory> {
    let MacBsdObservation::Active(Ok(kernel)) = MacSystemProbe.bsd(pid, false) else {
        return Err(CefUnavailableCategory::Admission);
    };
    let executable = MacSystemProbe
        .executable(pid)
        .map_err(|_| CefUnavailableCategory::Admission)?;
    Ok(MacProcessIdentity {
        pid,
        parent_pid: kernel.parent_pid,
        started_at: kernel.started_at,
        process_group: kernel.process_group,
        executable,
    })
}

pub(super) fn classify_signal_result(
    result: i32,
    errno: Option<i32>,
) -> Result<MacSignalResult, CefUnavailableCategory> {
    if result == 0 {
        Ok(MacSignalResult::Sent)
    } else if errno == Some(libc::ESRCH) {
        Ok(MacSignalResult::Stopped)
    } else {
        Err(CefUnavailableCategory::Reaper)
    }
}

pub(super) fn observe_with(
    probe: &impl MacProcessProbe,
    identity: &MacProcessIdentity,
) -> MacProcessObservation {
    match probe.bsd(identity.pid, true) {
        MacBsdObservation::Zombie => MacProcessObservation::Stopped,
        MacBsdObservation::Active(Ok(current))
            if current == MacKernelIdentity::from_identity(identity) =>
        {
            MacProcessObservation::Alive
        }
        MacBsdObservation::Active(Ok(_)) => MacProcessObservation::Stopped,
        MacBsdObservation::Active(Err(())) => MacProcessObservation::Unknown,
        MacBsdObservation::Unavailable => fallback(probe, identity.pid),
    }
}

pub(super) fn revalidate_with(
    probe: &impl MacProcessProbe,
    identity: &MacProcessIdentity,
) -> MacSignalObservation {
    match probe.bsd(identity.pid, false) {
        MacBsdObservation::Zombie => return MacSignalObservation::Stopped,
        MacBsdObservation::Active(Ok(current))
            if current == MacKernelIdentity::from_identity(identity) => {}
        MacBsdObservation::Active(Ok(_)) => return MacSignalObservation::Stopped,
        MacBsdObservation::Active(Err(())) => return MacSignalObservation::Unknown,
        MacBsdObservation::Unavailable => return fallback_signal(probe, identity.pid),
    }
    match probe.executable(identity.pid) {
        Ok(executable) if executable == identity.executable => MacSignalObservation::Ready,
        Ok(_) => MacSignalObservation::Stopped,
        Err(()) => MacSignalObservation::Unknown,
    }
}

fn fallback(probe: &impl MacProcessProbe, pid: u32) -> MacProcessObservation {
    match (probe.wait(pid), probe.existence(pid)) {
        (MacWaitObservation::Reapable, _) | (_, MacExistenceObservation::Missing) => {
            MacProcessObservation::Stopped
        }
        _ => MacProcessObservation::Unknown,
    }
}

fn fallback_signal(probe: &impl MacProcessProbe, pid: u32) -> MacSignalObservation {
    match fallback(probe, pid) {
        MacProcessObservation::Stopped => MacSignalObservation::Stopped,
        MacProcessObservation::Alive | MacProcessObservation::Unknown => {
            MacSignalObservation::Unknown
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct MacKernelIdentity {
    pub(super) parent_pid: u32,
    pub(super) started_at: u64,
    pub(super) process_group: u32,
}

impl MacKernelIdentity {
    pub(super) fn from_identity(identity: &MacProcessIdentity) -> Self {
        Self {
            parent_pid: identity.parent_pid,
            started_at: identity.started_at,
            process_group: identity.process_group,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum MacBsdObservation {
    Active(Result<MacKernelIdentity, ()>),
    Zombie,
    Unavailable,
}

#[derive(Clone, Copy)]
pub(super) enum MacWaitObservation {
    Reapable,
    NotReapable,
}

#[derive(Clone, Copy)]
pub(super) enum MacExistenceObservation {
    Present,
    Missing,
    Unknown,
}

pub(super) trait MacProcessProbe {
    fn bsd(&self, pid: u32, include_zombies: bool) -> MacBsdObservation;
    fn wait(&self, pid: u32) -> MacWaitObservation;
    fn existence(&self, pid: u32) -> MacExistenceObservation;
    fn executable(&self, pid: u32) -> Result<PathBuf, ()>;
}
