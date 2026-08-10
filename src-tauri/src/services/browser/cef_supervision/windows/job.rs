use super::super::CefUnavailableCategory;
use super::handle::OwnedHandle;
use super::identity::WindowsProcessIdentity;
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, IsProcessInJob,
    JobObjectBasicAccountingInformation, JobObjectExtendedLimitInformation,
    QueryInformationJobObject, SetInformationJobObject, JOBOBJECT_BASIC_ACCOUNTING_INFORMATION,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};

pub(in crate::services::browser) struct WindowsJobGuard {
    handle: OwnedHandle,
}

impl WindowsJobGuard {
    pub(in crate::services::browser) fn new() -> Result<Self, CefUnavailableCategory> {
        let handle =
            OwnedHandle::new(unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) })?;
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let set = unsafe {
            SetInformationJobObject(
                handle.raw(),
                JobObjectExtendedLimitInformation,
                std::ptr::addr_of!(limits).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
        };
        if set == 0 {
            return Err(CefUnavailableCategory::Admission);
        }
        Ok(Self { handle })
    }

    pub(in crate::services::browser) fn assign(
        &self,
        process: &WindowsProcessIdentity,
    ) -> Result<(), CefUnavailableCategory> {
        if unsafe { AssignProcessToJobObject(self.handle.raw(), process.raw_handle()) } == 0 {
            Err(CefUnavailableCategory::Admission)
        } else {
            Ok(())
        }
    }

    pub(in crate::services::browser) fn contains(
        &self,
        process: &WindowsProcessIdentity,
    ) -> Result<bool, CefUnavailableCategory> {
        let mut contained = 0;
        if unsafe { IsProcessInJob(process.raw_handle(), self.handle.raw(), &mut contained) } == 0 {
            Err(CefUnavailableCategory::Admission)
        } else {
            Ok(contained != 0)
        }
    }

    pub(in crate::services::browser) fn is_empty(&self) -> Result<bool, CefUnavailableCategory> {
        let mut accounting = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        query_job(
            &self.handle,
            JobObjectBasicAccountingInformation,
            &mut accounting,
        )?;
        Ok(accounting.TotalProcesses == 0 && accounting.ActiveProcesses == 0)
    }

    pub(in crate::services::browser) fn has_only_kill_on_close(
        &self,
    ) -> Result<bool, CefUnavailableCategory> {
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        query_job(&self.handle, JobObjectExtendedLimitInformation, &mut limits)?;
        Ok(limits.BasicLimitInformation.LimitFlags == JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE)
    }

    pub(super) fn into_raw(self) -> windows_sys::Win32::Foundation::HANDLE {
        self.handle.into_raw()
    }
}

impl std::fmt::Debug for WindowsJobGuard {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WindowsJobGuard([redacted])")
    }
}

fn query_job<T>(
    handle: &OwnedHandle,
    class: i32,
    output: &mut T,
) -> Result<(), CefUnavailableCategory> {
    if unsafe {
        QueryInformationJobObject(
            handle.raw(),
            class,
            std::ptr::from_mut(output).cast(),
            std::mem::size_of::<T>() as u32,
            std::ptr::null_mut(),
        )
    } == 0
    {
        Err(CefUnavailableCategory::Admission)
    } else {
        Ok(())
    }
}
