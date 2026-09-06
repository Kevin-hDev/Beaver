//! Practical monitored allowance, not an operating-system disk quota.
use std::path::Path;
pub(super) const WARNING_BYTES: u64 = 1024 * 1024 * 1024;
pub(super) const PUBLICATION_RESERVE: u64 = 1024 * 1024 * 1024;
pub(super) const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(250);

#[derive(Clone, Copy)]
pub(super) struct DiskPolicy {
    pub warning_bytes: u64,
    pub reserve_bytes: u64,
    pub poll_interval: std::time::Duration,
}
impl Default for DiskPolicy {
    fn default() -> Self {
        Self {
            warning_bytes: WARNING_BYTES,
            reserve_bytes: PUBLICATION_RESERVE,
            poll_interval: POLL_INTERVAL,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::services::extensions) struct StorageAllowance {
    pub approved_total_bytes: u64,
    pub confirmation_used: bool,
}
impl Default for StorageAllowance {
    fn default() -> Self {
        Self::new(DiskPolicy::default())
    }
}
impl StorageAllowance {
    pub(super) fn new(policy: DiskPolicy) -> Self {
        Self {
            approved_total_bytes: policy.warning_bytes,
            confirmation_used: false,
        }
    }
    pub(super) fn check(
        &self,
        occupied: u64,
        free: u64,
        policy: DiskPolicy,
    ) -> Result<(), super::InstallInterruption> {
        use super::InstallInterruption::{Confirmation, InsufficientSpace};
        if free <= policy.reserve_bytes {
            return Err(InsufficientSpace);
        }
        if occupied > self.approved_total_bytes {
            return Err(if self.confirmation_used {
                InsufficientSpace
            } else {
                Confirmation
            });
        }
        Ok(())
    }
    pub(super) fn approve(
        &mut self,
        occupied: u64,
        free: u64,
        policy: DiskPolicy,
    ) -> Result<(), super::InstallInterruption> {
        if self.confirmation_used {
            return Err(super::InstallInterruption::Failed);
        }
        let available = free
            .checked_sub(policy.reserve_bytes)
            .filter(|bytes| *bytes > 0)
            .ok_or(super::InstallInterruption::InsufficientSpace)?;
        self.approved_total_bytes = occupied
            .checked_add(available)
            .ok_or(super::InstallInterruption::InsufficientSpace)?;
        self.confirmation_used = true;
        Ok(())
    }
}

#[allow(clippy::unnecessary_cast)] // statvfs field widths differ across supported Unix targets.
pub(super) fn free_bytes(path: &Path) -> Result<u64, String> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::CString::new(path.as_os_str().as_bytes())
            .map_err(|_| super::limits::INVALID)?;
        let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        // SAFETY: CString is terminated and stat points to writable storage.
        if unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) } != 0 {
            return Err(super::limits::UNAVAILABLE.into());
        }
        let stat = unsafe { stat.assume_init() };
        (stat.f_bavail as u64)
            .checked_mul(stat.f_frsize as u64)
            .ok_or(super::limits::UNAVAILABLE.into())
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut available = 0_u64;
        let ok = unsafe {
            windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                path.as_ptr(),
                &mut available,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            Err(super::limits::UNAVAILABLE.into())
        } else {
            Ok(available)
        }
    }
}
