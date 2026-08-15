impl NativeGatedProcess {
    pub(crate) fn identity(&self) -> OwnedProcessIdentity {
        self.identity
    }

    pub(crate) fn open_gate(&mut self) -> Result<(), OllamaProcessError> {
        let mut gate = self.gate.take().ok_or(OllamaProcessError::Gate)?;
        gate.write_all(&[1]).map_err(|_| OllamaProcessError::Gate)?;
        self.opened = true;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn close_gate_for_test(&mut self) {
        self.test_gate_keepalive = self.gate.take();
        self.gate = std::fs::File::open("/dev/null").ok();
    }

    pub(crate) fn revalidate(&self, executable: u128) -> Result<(), OllamaProcessError> {
        let current = if self.opened {
            OwnedProcess::identity(self.identity.pid)
        } else {
            OwnedProcess::identity_with_executable(self.identity.pid, executable)
        }
        .map_err(|_| OllamaProcessError::Identity)?;
        if current != self.identity || current.executable != executable {
            return Err(OllamaProcessError::Identity);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn force_identity_change_for_test(&mut self) {
        self.identity.native_start_time ^= 1;
    }

    pub(crate) fn wait_for_executable(
        &mut self,
        executable: u128,
        deadline: Instant,
    ) -> Result<(), OllamaProcessError> {
        while Instant::now() < deadline {
            let current = OwnedProcess::identity(self.identity.pid)
                .map_err(|_| OllamaProcessError::Identity)?;
            if current.pid != self.identity.pid
                || current.native_scope != self.identity.native_scope
                || current.native_start_time != self.identity.native_start_time
            {
                return Err(OllamaProcessError::Identity);
            }
            if current.executable == executable {
                self.exec_link.take();
                return Ok(());
            }
            std::thread::yield_now();
        }
        Err(OllamaProcessError::Gate)
    }

    #[cfg(test)]
    pub(crate) fn exec_link_exists_for_test(&self) -> bool {
        self.exec_link
            .as_ref()
            .is_some_and(|link| link.path().exists())
    }

    pub(crate) fn terminate(&mut self) -> Result<(), OllamaProcessError> {
        if self.reaped {
            return Ok(());
        }
        self.gate.take();
        let group = unsafe { libc::kill(-self.pid, libc::SIGTERM) };
        let group_error = std::io::Error::last_os_error().raw_os_error();
        let process = unsafe { libc::kill(self.pid, libc::SIGTERM) };
        let process_error = std::io::Error::last_os_error().raw_os_error();
        if group == 0 || process == 0 {
            return Ok(());
        }
        if group_error == Some(libc::ESRCH) && process_error == Some(libc::ESRCH) {
            return Ok(());
        }
        Err(OllamaProcessError::Gate)
    }

    pub(crate) fn reap(&mut self, deadline: Instant) -> Result<(), OllamaProcessError> {
        #[cfg(test)]
        if self.force_reap_failure {
            return Err(OllamaProcessError::Reap);
        }
        if self.reaped {
            return Ok(());
        }
        while Instant::now() < deadline {
            match wait_nonblocking(self.pid)? {
                Some(_) => {
                    crate::services::owned_process::release(self.identity.pid);
                    self.reaped = true;
                    return Ok(());
                }
                None => std::thread::yield_now(),
            }
        }
        unsafe {
            libc::kill(-self.pid, libc::SIGKILL);
            libc::kill(self.pid, libc::SIGKILL);
        }
        wait_blocking(self.pid)?;
        crate::services::owned_process::release(self.identity.pid);
        self.reaped = true;
        Ok(())
    }

    pub(crate) fn terminate_and_reap(
        &mut self,
        deadline: Instant,
    ) -> Result<(), OllamaProcessError> {
        self.terminate()?;
        self.reap(deadline)
    }

    #[cfg(test)]
    pub(crate) fn force_reap_failure_for_test(&mut self) {
        self.force_reap_failure = true;
    }
}
