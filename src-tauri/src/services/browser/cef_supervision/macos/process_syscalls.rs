use super::process_state::{
    MacBsdObservation, MacExistenceObservation, MacKernelIdentity, MacProcessProbe,
    MacWaitObservation,
};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

const MAX_EXECUTABLE_BYTES: usize = 4_096;

pub(super) struct MacSystemProbe;

impl MacProcessProbe for MacSystemProbe {
    fn bsd(&self, pid: u32, include_zombies: bool) -> MacBsdObservation {
        if pid == 0 || pid > i32::MAX as u32 {
            return MacBsdObservation::Unavailable;
        }
        let mut info = unsafe { std::mem::zeroed::<libc::proc_bsdinfo>() };
        let size = std::mem::size_of::<libc::proc_bsdinfo>();
        let read = unsafe {
            libc::proc_pidinfo(
                pid as i32,
                libc::PROC_PIDTBSDINFO,
                i32::from(include_zombies) as u64,
                (&mut info as *mut libc::proc_bsdinfo).cast(),
                size as i32,
            )
        };
        if read != size as i32 || info.pbi_pid != pid {
            return MacBsdObservation::Unavailable;
        }
        if info.pbi_status == libc::SZOMB {
            return MacBsdObservation::Zombie;
        }
        MacBsdObservation::Active(kernel_from_info(pid, &info))
    }

    fn wait(&self, pid: u32) -> MacWaitObservation {
        let mut exit_info = unsafe { std::mem::zeroed::<libc::siginfo_t>() };
        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                pid as libc::id_t,
                &mut exit_info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if result == 0 && unsafe { exit_info.si_pid() } == pid as libc::pid_t {
            MacWaitObservation::Reapable
        } else {
            MacWaitObservation::NotReapable
        }
    }

    fn existence(&self, pid: u32) -> MacExistenceObservation {
        if unsafe { libc::kill(pid as i32, 0) } == 0 {
            MacExistenceObservation::Present
        } else if std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
            MacExistenceObservation::Missing
        } else {
            MacExistenceObservation::Unknown
        }
    }

    fn executable(&self, pid: u32) -> Result<PathBuf, ()> {
        let mut buffer = Box::new([0_u8; MAX_EXECUTABLE_BYTES]);
        let read = unsafe {
            libc::proc_pidpath(
                pid as i32,
                buffer.as_mut_ptr().cast(),
                MAX_EXECUTABLE_BYTES as u32,
            )
        };
        if read <= 0 || read as usize >= MAX_EXECUTABLE_BYTES {
            return Err(());
        }
        let bytes = &buffer[..read as usize];
        if bytes.contains(&0) {
            return Err(());
        }
        dunce::canonicalize(PathBuf::from(OsStr::from_bytes(bytes))).map_err(|_| ())
    }
}

pub(super) fn signal_group_raw(process_group: i32) -> (i32, Option<i32>) {
    let result = unsafe { libc::kill(-process_group, libc::SIGKILL) };
    let errno = (result != 0)
        .then(|| std::io::Error::last_os_error().raw_os_error())
        .flatten();
    (result, errno)
}

fn kernel_from_info(pid: u32, info: &libc::proc_bsdinfo) -> Result<MacKernelIdentity, ()> {
    let process_group = unsafe { libc::getpgid(pid as i32) };
    let started_at = info
        .pbi_start_tvsec
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(info.pbi_start_tvusec))
        .filter(|value| *value != 0)
        .ok_or(())?;
    if process_group <= 0 {
        return Err(());
    }
    Ok(MacKernelIdentity {
        parent_pid: info.pbi_ppid,
        started_at,
        process_group: process_group as u32,
    })
}
