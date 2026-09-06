use super::{OwnedProcessError, ProcessKind};

pub(crate) struct OwnedProcessScope {
    #[cfg(windows)]
    job: super::platform::DedicatedJob,
}

impl OwnedProcessScope {
    pub(super) async fn spawn_tokio(
        command: &mut tokio::process::Command,
        kind: ProcessKind,
    ) -> Result<(tokio::process::Child, Self), OwnedProcessError> {
        #[cfg(not(windows))]
        {
            let child = super::OwnedProcess::spawn_tokio(command, kind).await?;
            Ok((child, Self {}))
        }
        #[cfg(windows)]
        {
            super::process_tree::configure_tokio(command);
            crate::services::background_command::configure_tokio_with_extra_flags(
                command,
                windows_sys::Win32::System::Threading::CREATE_SUSPENDED,
            );
            let job = super::platform::DedicatedJob::new()?;
            let mut child = command
                .spawn()
                .map_err(|error| OwnedProcessError::Spawn(error.kind()))?;
            let Some(pid) = child.id() else {
                super::process_tree::terminate_tokio(&mut child, kind).await;
                return Err(OwnedProcessError::Admission);
            };
            if job.admit(pid).is_err() {
                let _ = child.start_kill();
                let _ = child.wait().await;
                return Err(OwnedProcessError::Admission);
            }
            if super::platform::resume_suspended_process(pid).is_err() {
                let _ = job.terminate();
                let _ = child.wait().await;
                return Err(OwnedProcessError::Admission);
            }
            Ok((child, Self { job }))
        }
    }

    pub(super) async fn spawn_tokio_with_owner_pipe(
        command: &mut tokio::process::Command,
        kind: ProcessKind,
    ) -> Result<(tokio::process::Child, Self), OwnedProcessError> {
        #[cfg(target_os = "linux")]
        {
            command.process_group(0);
            let child = super::OwnedProcess::spawn_tokio_configured(command, kind).await?;
            Ok((child, Self {}))
        }
        #[cfg(not(target_os = "linux"))]
        Self::spawn_tokio(command, kind).await
    }

    /// Identity must be checked by the job that admitted this process, not the
    /// unrelated global job. Windows keeps these containment authorities distinct.
    pub(crate) fn identity(
        &self,
        pid: u32,
    ) -> Result<super::OwnedProcessIdentity, OwnedProcessError> {
        #[cfg(windows)]
        return self.job.identity(pid);
        #[cfg(not(windows))]
        super::OwnedProcess::identity(pid)
    }

    pub(crate) fn terminate(&self) -> bool {
        #[cfg(windows)]
        return self.job.terminate().is_ok();
        #[cfg(not(windows))]
        true
    }

    #[cfg(windows)]
    pub(crate) fn is_empty(&self) -> bool {
        self.job.is_empty().unwrap_or(false)
    }

    #[cfg(all(test, windows))]
    pub(crate) fn contains(&self, pid: u32) -> bool {
        self.job.contains(pid)
    }
}

#[cfg(all(test, windows))]
#[path = "owned_process_scope_windows_tests.rs"]
mod windows_tests;
