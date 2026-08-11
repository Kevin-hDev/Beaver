use super::super::CefUnavailableCategory;
use super::identity::WindowsProcessIdentity;
use super::job::WindowsJobGuard;

pub(in crate::services::browser) struct WindowsConfinement {
    process: WindowsProcessIdentity,
    job: WindowsJobGuard,
}

impl WindowsConfinement {
    pub(in crate::services::browser) fn establish(
        process: WindowsProcessIdentity,
    ) -> Result<Self, CefUnavailableCategory> {
        let job = match WindowsJobGuard::new() {
            Ok(job) => job,
            Err(error) => {
                let _ = process.terminate();
                return Err(error);
            }
        };
        if job.assign(&process).is_err() || job.contains(&process) != Ok(true) {
            let _ = process.terminate();
            return Err(CefUnavailableCategory::Admission);
        }
        Ok(Self { process, job })
    }

    pub(super) fn into_raw(self) -> WindowsConfinementParts {
        WindowsConfinementParts {
            process: self.process.into_parts(),
            job: self.job.into_raw(),
        }
    }
}

impl std::fmt::Debug for WindowsConfinement {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WindowsConfinement([redacted])")
    }
}

pub(super) struct WindowsConfinementParts {
    pub(super) process: super::identity::WindowsProcessParts,
    pub(super) job: windows_sys::Win32::Foundation::HANDLE,
}
