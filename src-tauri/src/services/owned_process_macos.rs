use super::OwnedProcessError;
use std::sync::{Mutex, OnceLock};

const MAX_OWNED_PROCESSES: usize = 64;
static WATCHDOG: OnceLock<MacProcessWatchdog> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MacProcessIdentity {
    pid: u32,
    parent_pid: u32,
    process_group: u32,
    started_at: u64,
}

struct MacProcessWatchdog {
    slots: Mutex<[Option<MacProcessIdentity>; MAX_OWNED_PROCESSES]>,
}

impl MacProcessWatchdog {
    fn global() -> &'static Self {
        WATCHDOG.get_or_init(|| Self {
            slots: Mutex::new([None; MAX_OWNED_PROCESSES]),
        })
    }

    fn admit(&self, pid: u32) -> Result<(), OwnedProcessError> {
        let identity = read_identity(pid)?;
        if identity.parent_pid != std::process::id() || identity.process_group != pid {
            return Err(OwnedProcessError::Admission);
        }
        let mut slots = self
            .slots
            .lock()
            .map_err(|_| OwnedProcessError::Admission)?;
        for slot in slots.iter_mut() {
            if slot.is_some_and(|current| !current.is_current()) {
                *slot = None;
            }
        }
        let slot = slots
            .iter_mut()
            .find(|slot| slot.is_none())
            .ok_or(OwnedProcessError::Admission)?;
        *slot = Some(identity);
        Ok(())
    }

    fn release(&self, pid: u32) {
        let Ok(mut slots) = self.slots.lock() else {
            return;
        };
        if let Some(slot) = slots
            .iter_mut()
            .find(|slot| slot.is_some_and(|identity| identity.pid == pid))
        {
            *slot = None;
        }
    }

    fn identity(&self, pid: u32) -> Option<MacProcessIdentity> {
        self.slots
            .lock()
            .ok()?
            .iter()
            .flatten()
            .find(|identity| identity.pid == pid)
            .copied()
    }
}

impl MacProcessIdentity {
    fn is_current(self) -> bool {
        read_identity(self.pid).is_ok_and(|current| current == self)
    }
}

pub(super) fn admit(pid: u32) -> Result<(), OwnedProcessError> {
    MacProcessWatchdog::global().admit(pid)
}

pub(super) fn release(pid: u32) {
    MacProcessWatchdog::global().release(pid);
}

pub(super) fn signal_is_safe(pid: u32) -> bool {
    MacProcessWatchdog::global()
        .identity(pid)
        .is_none_or(MacProcessIdentity::is_current)
}

#[cfg(test)]
pub(super) fn is_confined(pid: u32) -> bool {
    MacProcessWatchdog::global()
        .identity(pid)
        .is_some_and(MacProcessIdentity::is_current)
}

fn read_identity(pid: u32) -> Result<MacProcessIdentity, OwnedProcessError> {
    if pid < 2 || pid > i32::MAX as u32 {
        return Err(OwnedProcessError::Admission);
    }
    let mut info = unsafe { std::mem::zeroed::<libc::proc_bsdinfo>() };
    let size = std::mem::size_of::<libc::proc_bsdinfo>();
    let read = unsafe {
        libc::proc_pidinfo(
            pid as i32,
            libc::PROC_PIDTBSDINFO,
            0,
            (&mut info as *mut libc::proc_bsdinfo).cast(),
            size as i32,
        )
    };
    if read != size as i32 || info.pbi_pid != pid || info.pbi_status == libc::SZOMB {
        return Err(OwnedProcessError::Admission);
    }
    let process_group = unsafe { libc::getpgid(pid as i32) };
    let started_at = info
        .pbi_start_tvsec
        .checked_mul(1_000_000)
        .and_then(|seconds| seconds.checked_add(info.pbi_start_tvusec))
        .filter(|started| *started != 0)
        .ok_or(OwnedProcessError::Admission)?;
    if process_group <= 0 {
        return Err(OwnedProcessError::Admission);
    }
    Ok(MacProcessIdentity {
        pid,
        parent_pid: info.pbi_ppid,
        process_group: process_group as u32,
        started_at,
    })
}
