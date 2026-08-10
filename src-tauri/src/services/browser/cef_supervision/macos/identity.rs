use super::super::CefUnavailableCategory;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const MAX_EXECUTABLE_BYTES: usize = 4_096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::services::browser) struct MacProcessIdentity {
    pub(super) pid: u32,
    pub(super) parent_pid: u32,
    pub(super) started_at: u64,
    pub(super) process_group: u32,
    pub(super) executable: PathBuf,
}

impl MacProcessIdentity {
    pub(in crate::services::browser) fn read(pid: u32) -> Result<Self, CefUnavailableCategory> {
        let kernel = kernel_identity(pid)?;
        Ok(Self {
            pid,
            parent_pid: kernel.parent_pid,
            started_at: kernel.started_at,
            process_group: kernel.process_group,
            executable: executable(pid)?,
        })
    }

    pub(in crate::services::browser) fn validate(
        pid: u32,
        parent_pid: u32,
        started_at: u64,
        process_group: u32,
        executable: &Path,
    ) -> Result<Self, CefUnavailableCategory> {
        let identity = Self::read(pid)?;
        let expected =
            dunce::canonicalize(executable).map_err(|_| CefUnavailableCategory::Admission)?;
        if parent_pid == 0
            || started_at == 0
            || process_group != pid
            || identity.parent_pid != parent_pid
            || identity.started_at != started_at
            || identity.process_group != process_group
            || identity.executable != expected
        {
            Err(CefUnavailableCategory::Admission)
        } else {
            Ok(identity)
        }
    }

    pub(super) fn revalidate(&self) -> Result<(), CefUnavailableCategory> {
        (Self::read(self.pid)? == *self)
            .then_some(())
            .ok_or(CefUnavailableCategory::Reaper)
    }

    pub(super) fn kill_group(&self) -> Result<(), CefUnavailableCategory> {
        self.revalidate()?;
        if unsafe { libc::kill(-(self.process_group as i32), libc::SIGKILL) } == 0 {
            Ok(())
        } else {
            Err(CefUnavailableCategory::Reaper)
        }
    }

    pub(super) fn is_alive(&self) -> Result<bool, CefUnavailableCategory> {
        match kernel_identity(self.pid) {
            Ok(current) => Ok(current == self.kernel_identity()),
            Err(_) => {
                let result = unsafe { libc::kill(self.pid as i32, 0) };
                if result == 0 {
                    Err(CefUnavailableCategory::Reaper)
                } else if std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
                    Ok(false)
                } else {
                    Err(CefUnavailableCategory::Reaper)
                }
            }
        }
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn test_pid(&self) -> u32 {
        self.pid
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn test_parent_pid(&self) -> u32 {
        self.parent_pid
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn test_started_at(&self) -> u64 {
        self.started_at
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn test_process_group(&self) -> u32 {
        self.process_group
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn test_executable(&self) -> &Path {
        &self.executable
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MacKernelIdentity {
    parent_pid: u32,
    started_at: u64,
    process_group: u32,
}

fn kernel_identity(pid: u32) -> Result<MacKernelIdentity, CefUnavailableCategory> {
    if pid == 0 || pid > i32::MAX as u32 {
        return Err(CefUnavailableCategory::Admission);
    }
    let info = bsd_info(pid)?;
    let process_group = unsafe { libc::getpgid(pid as i32) };
    if process_group <= 0 {
        return Err(CefUnavailableCategory::Admission);
    }
    let started_at = info
        .pbi_start_tvsec
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(info.pbi_start_tvusec))
        .filter(|value| *value != 0)
        .ok_or(CefUnavailableCategory::Admission)?;
    Ok(MacKernelIdentity {
        parent_pid: info.pbi_ppid,
        started_at,
        process_group: process_group as u32,
    })
}

fn bsd_info(pid: u32) -> Result<libc::proc_bsdinfo, CefUnavailableCategory> {
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
    (read == size as i32 && info.pbi_pid == pid)
        .then_some(info)
        .ok_or(CefUnavailableCategory::Admission)
}

fn executable(pid: u32) -> Result<PathBuf, CefUnavailableCategory> {
    let mut buffer = Box::new([0_u8; MAX_EXECUTABLE_BYTES]);
    let read = unsafe {
        libc::proc_pidpath(
            pid as i32,
            buffer.as_mut_ptr().cast(),
            MAX_EXECUTABLE_BYTES as u32,
        )
    };
    if read <= 0 || read as usize >= MAX_EXECUTABLE_BYTES {
        return Err(CefUnavailableCategory::Admission);
    }
    let bytes = &buffer[..read as usize];
    if bytes.contains(&0) {
        return Err(CefUnavailableCategory::Admission);
    }
    dunce::canonicalize(PathBuf::from(OsStr::from_bytes(bytes)))
        .map_err(|_| CefUnavailableCategory::Admission)
}
