use crate::services::terminal::exit_wait::{reap_child, wait_for_exit_code, ExitPoll};
use crate::services::terminal::limits::MAX_PTY_WRITE_BYTES;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

type SharedChild = Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>;

pub struct PtySession {
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    child: SharedChild,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
}

#[derive(Clone)]
pub(crate) struct PtyChildStatus(SharedChild);

impl PtyChildStatus {
    pub(crate) fn exit_code(&self) -> Option<u32> {
        wait_for_exit_code(|| match self.0.try_lock() {
            Ok(mut child) => match child.try_wait() {
                Ok(Some(status)) => ExitPoll::Exited(Some(status.exit_code())),
                Ok(None) => ExitPoll::Running,
                Err(_) => ExitPoll::Failed,
            },
            Err(std::sync::TryLockError::WouldBlock) => ExitPoll::Running,
            Err(std::sync::TryLockError::Poisoned(_)) => ExitPoll::Failed,
        })
    }
}

impl PtySession {
    pub fn spawn(
        cwd: Option<&Path>,
        cols: u16,
        rows: u16,
    ) -> Result<(Self, Box<dyn Read + Send>), String> {
        validate_size(cols, rows)?;
        let pair = NativePtySystem::default()
            .openpty(pty_size(cols, rows))
            .map_err(|_| terminal_error())?;
        let mut command = terminal_command()?;
        if let Some(directory) = validated_cwd(cwd)? {
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
                master: Some(pair.master),
                child: Arc::new(Mutex::new(child)),
                writer: Mutex::new(Some(writer)),
            },
            reader,
        ))
    }

    #[cfg(test)]
    pub(in crate::services::terminal) fn terminal_command_for_test(
    ) -> Result<CommandBuilder, String> {
        terminal_command()
    }

    pub fn write(&self, data: &[u8]) -> Result<(), String> {
        if data.len() > MAX_PTY_WRITE_BYTES {
            return Err("terminal-write-too-large".to_string());
        }
        let mut writer = self.writer.lock().map_err(|_| terminal_error())?;
        let writer = writer.as_mut().ok_or_else(terminal_error)?;
        writer.write_all(data).map_err(|_| terminal_error())?;
        writer.flush().map_err(|_| terminal_error())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        validate_size(cols, rows)?;
        self.master
            .as_ref()
            .ok_or_else(terminal_error)?
            .resize(pty_size(cols, rows))
            .map_err(|_| terminal_error())
    }

    #[cfg(test)]
    pub fn kill(&mut self) -> Result<(), String> {
        self.shutdown().map(|_| ())
    }

    pub(crate) fn process_id(&self) -> Option<u32> {
        self.child.lock().ok()?.process_id()
    }

    pub(crate) fn child_status(&self) -> PtyChildStatus {
        PtyChildStatus(Arc::clone(&self.child))
    }

    pub(crate) fn shutdown(&mut self) -> Result<u32, String> {
        let _ = self.close_input();
        if let Some(code) = self.child_status().exit_code() {
            return Ok(code);
        }
        let pid = self.process_id().ok_or_else(terminal_error)?;
        crate::services::process_tree::kill(
            pid,
            crate::services::process_tree::ProcessKind::Terminal,
        );
        // Le maître part avant l'attente. Tant qu'il reste ouvert et que
        // personne ne le draine, le noyau retient le shell dans sa sortie : il
        // n'atteint jamais l'état zombie et wait() ne rend pas la main. Or ce
        // maître n'est refermé qu'à la fin du Drop, c'est-à-dire après cette
        // attente — chacun attendait l'autre.
        drop(self.master.take());
        let mut child = self.child.lock().map_err(|_| terminal_error())?;
        if child.try_wait().map_err(|_| terminal_error())?.is_none() {
            child.kill().map_err(|_| terminal_error())?;
        }
        reap_child(|| match child.try_wait() {
            Ok(Some(status)) => ExitPoll::Exited(Some(status.exit_code())),
            Ok(None) => ExitPoll::Running,
            Err(_) => ExitPoll::Failed,
        })
        .ok_or_else(terminal_error)
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

fn terminal_command() -> Result<CommandBuilder, String> {
    let shell = super::super::shell_helper::terminal_shell_executable()?;
    #[cfg(all(target_os = "linux", not(test)))]
    let mut command = {
        let current = dunce::canonicalize(std::env::current_exe().map_err(|_| terminal_error())?)
            .map_err(|_| terminal_error())?;
        let mut command = CommandBuilder::new(current);
        command.arg(super::super::shell_helper::ROLE_FLAG);
        command.arg(std::process::id().to_string());
        command.arg("--");
        command.arg(shell);
        command
    };
    #[cfg(any(not(target_os = "linux"), test))]
    let mut command = CommandBuilder::new(shell);
    command.arg("-l");
    command.env("TERM", "xterm-256color");
    Ok(command)
}

fn validate_size(cols: u16, rows: u16) -> Result<(), String> {
    if cols == 0 || rows == 0 {
        Err("terminal-size-invalid".to_string())
    } else {
        Ok(())
    }
}

fn validated_cwd(cwd: Option<&Path>) -> Result<Option<&Path>, String> {
    let Some(directory) = cwd else {
        return Ok(None);
    };
    if !directory.is_absolute() || !directory.is_dir() {
        Err("terminal-cwd-invalid".to_string())
    } else {
        Ok(Some(directory))
    }
}

fn terminal_error() -> String {
    "terminal-error".to_string()
}
