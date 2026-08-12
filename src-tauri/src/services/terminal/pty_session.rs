use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

type SharedChild = Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>;

pub struct PtySession {
    master: Box<dyn portable_pty::MasterPty + Send>,
    child: SharedChild,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
}

#[derive(Clone)]
pub(super) struct PtyChildStatus(SharedChild);

impl PtyChildStatus {
    pub(super) fn exit_code(&self) -> Option<u32> {
        self.0
            .lock()
            .ok()?
            .try_wait()
            .ok()
            .flatten()
            .map(|status| status.exit_code())
    }
}

impl PtySession {
    pub fn spawn(
        cwd: Option<&str>,
        cols: u16,
        rows: u16,
    ) -> Result<(Self, Box<dyn Read + Send>), String> {
        if cols == 0 || rows == 0 {
            return Err("terminal-size-invalid".to_string());
        }
        let pty_system = NativePtySystem::default();
        let pair = pty_system
            .openpty(pty_size(cols, rows))
            .map_err(|_| terminal_error())?;
        let shell = default_shell()?;
        let mut command = CommandBuilder::new(&shell);

        #[cfg(unix)]
        command.arg("-l");

        command.env("TERM", "xterm-256color");
        if std::env::var("EDITOR").is_ok_and(|editor| editor.contains("vi")) {
            command.env("EDITOR", "");
        }
        if let Some(directory) = cwd {
            let path = std::path::Path::new(directory);
            if !path.is_absolute() || !path.is_dir() {
                return Err("terminal-cwd-invalid".to_string());
            }
            command.cwd(directory);
        }

        let mut child = pair
            .slave
            .spawn_command(command)
            .map_err(|_| terminal_error())?;
        let pid = child.process_id().ok_or_else(terminal_error)?;
        if crate::services::owned_process::OwnedProcess::adopt_existing(pid).is_err() {
            crate::services::process_tree::kill(
                pid,
                crate::services::process_tree::ProcessKind::Terminal,
            );
            let _ = child.kill();
            let _ = child.wait();
            return Err(terminal_error());
        }
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|_| terminal_error())?;
        let writer = pair.master.take_writer().map_err(|_| terminal_error())?;
        Ok((
            Self {
                master: pair.master,
                child: Arc::new(Mutex::new(child)),
                writer: Mutex::new(Some(writer)),
            },
            reader,
        ))
    }

    const MAX_WRITE_BYTES: usize = 65_536;

    pub fn write(&self, data: &[u8]) -> Result<(), String> {
        if data.len() > Self::MAX_WRITE_BYTES {
            return Err("terminal-write-too-large".to_string());
        }
        let mut writer = self.writer.lock().map_err(|_| terminal_error())?;
        let writer = writer.as_mut().ok_or_else(terminal_error)?;
        writer.write_all(data).map_err(|_| terminal_error())?;
        writer.flush().map_err(|_| terminal_error())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        if cols == 0 || rows == 0 {
            return Err("terminal-size-invalid".to_string());
        }
        self.master
            .resize(pty_size(cols, rows))
            .map_err(|_| terminal_error())
    }

    #[cfg(test)]
    pub fn kill(&mut self) -> Result<(), String> {
        self.shutdown().map(|_| ())
    }

    pub(super) fn process_id(&self) -> Option<u32> {
        self.child.lock().ok()?.process_id()
    }

    pub(super) fn child_status(&self) -> PtyChildStatus {
        PtyChildStatus(Arc::clone(&self.child))
    }

    pub(super) fn shutdown(&mut self) -> Result<u32, String> {
        #[cfg(test)]
        eprintln!("[terminal-test] shutdown: close input");
        let _ = self.close_input();
        if let Some(code) = self.child_status().exit_code() {
            #[cfg(test)]
            eprintln!("[terminal-test] shutdown: child already exited");
            return Ok(code);
        }
        let pid = self.process_id().ok_or_else(terminal_error)?;
        #[cfg(test)]
        eprintln!("[terminal-test] shutdown: terminate process tree");
        crate::services::process_tree::kill(
            pid,
            crate::services::process_tree::ProcessKind::Terminal,
        );
        #[cfg(test)]
        eprintln!("[terminal-test] shutdown: process tree returned");
        let mut child = self.child.lock().map_err(|_| terminal_error())?;
        if child.try_wait().map_err(|_| terminal_error())?.is_none() {
            #[cfg(test)]
            eprintln!("[terminal-test] shutdown: kill direct child");
            child.kill().map_err(|_| terminal_error())?;
        }
        let result = child
            .wait()
            .map(|status| status.exit_code())
            .map_err(|_| terminal_error());
        #[cfg(test)]
        eprintln!("[terminal-test] shutdown: child reaped");
        result
    }

    fn close_input(&self) -> Result<(), String> {
        let writer = self.writer.lock().map_err(|_| terminal_error())?.take();
        drop(writer);
        Ok(())
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

fn pty_size(cols: u16, rows: u16) -> PtySize {
    PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    }
}

fn default_shell() -> Result<String, String> {
    #[cfg(unix)]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let path = std::path::Path::new(&shell);
        if !path.is_absolute() || !path.is_file() {
            return Err("terminal-shell-invalid".to_string());
        }
        Ok(shell)
    }
    #[cfg(windows)]
    {
        Ok("powershell.exe".to_string())
    }
}

fn terminal_error() -> String {
    "terminal-error".to_string()
}
