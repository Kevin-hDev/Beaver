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
            Ok((child, Self { job }))
        }
    }

    pub(crate) fn terminate(&self) -> bool {
        #[cfg(windows)]
        return self.job.terminate().is_ok();
        #[cfg(not(windows))]
        true
    }

    #[cfg(all(test, windows))]
    pub(crate) fn contains(&self, pid: u32) -> bool {
        self.job.contains(pid)
    }
}

#[cfg(all(test, windows))]
#[path = "owned_process_scope_windows_tests.rs"]
mod windows_tests;
