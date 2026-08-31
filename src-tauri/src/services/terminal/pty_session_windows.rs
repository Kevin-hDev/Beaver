#[path = "pty_windows_console.rs"]
mod console;
#[path = "pty_windows_output.rs"]
mod output;

use console::PseudoConsole;
use output::{OutputControl, PtyReader};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::ExitStatus;
use std::sync::{Arc, Mutex};

type SharedChild = Arc<Mutex<windows_spawn::Child>>;

pub struct PtySession {
    child: SharedChild,
    writer: Mutex<Option<File>>,
    output: Arc<OutputControl>,
    console: PseudoConsole,
}

#[derive(Clone)]
pub(crate) struct PtyChildStatus(SharedChild);

impl PtyChildStatus {
    pub(crate) fn exit_code(&self) -> Option<u32> {
        self.0.lock().ok()?.try_wait().ok().flatten().map(exit_code)
    }
}

impl PtySession {
    pub fn spawn(
        cwd: Option<&Path>,
        cols: u16,
        rows: u16,
    ) -> Result<(Self, Box<dyn Read + Send>), String> {
        validate_size(cols, rows)?;
        let (console, input, output_file) = PseudoConsole::create(cols, rows)?;
        let powershell =
            crate::services::system_executable::powershell().map_err(|_| terminal_error())?;
        let mut command = windows_spawn::Command::new(powershell);
        command.env("TERM", "xterm-256color");
        if std::env::var("EDITOR").is_ok_and(|editor| editor.contains("vi")) {
            command.env("EDITOR", "");
        }
        if let Some(directory) = validated_cwd(cwd)? {
            command.current_dir(directory);
        }
        let child =
            crate::services::owned_process::OwnedProcess::spawn_conpty(&mut command, &console)
                .map_err(|_| terminal_error())?;
        let child = Arc::new(Mutex::new(child));
        let output = Arc::new(OutputControl::new(output_file));
        let reader = PtyReader::new(Arc::clone(&output), Arc::clone(&child));
        Ok((
            Self {
                child,
                writer: Mutex::new(Some(input)),
                output,
                console,
            },
            Box::new(reader),
        ))
    }

    const MAX_WRITE_BYTES: usize = 65_536;

    pub fn write(&self, data: &[u8]) -> Result<(), String> {
        if data.len() > Self::MAX_WRITE_BYTES {
            return Err("terminal-write-too-large".to_string());
        }
        let mut writer = self.writer.lock().map_err(|_| terminal_error())?;
        writer
            .as_mut()
            .ok_or_else(terminal_error)?
            .write_all(data)
            .map_err(|_| terminal_error())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        validate_size(cols, rows)?;
        self.console.resize(cols, rows)
    }

    #[cfg(test)]
    pub fn kill(&mut self) -> Result<(), String> {
        self.shutdown().map(|_| ())
    }

    pub(crate) fn process_id(&self) -> Option<u32> {
        self.child.lock().ok().map(|child| child.id())
    }

    pub(crate) fn child_status(&self) -> PtyChildStatus {
        PtyChildStatus(Arc::clone(&self.child))
    }

    pub(crate) fn shutdown(&mut self) -> Result<u32, String> {
        let pid = self.process_id().ok_or_else(terminal_error)?;
        crate::services::process_tree::kill(
            pid,
            crate::services::process_tree::ProcessKind::Terminal,
        );
        let mut child = self.child.lock().map_err(|_| terminal_error())?;
        if child.try_wait().map_err(|_| terminal_error())?.is_none() {
            // Tree termination is primary; root termination is the bounded
            // fallback. A concurrent exit can make this return access denied;
            // the following wait is the sole authority for final completion.
            let _ = child.kill();
        }
        let status = child.wait().map_err(|_| terminal_error())?;
        drop(child);
        self.output.close()?;
        self.close_input()?;
        self.console.close();
        Ok(exit_code(status))
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

fn exit_code(status: ExitStatus) -> u32 {
    status
        .code()
        .map(|code| u32::from_ne_bytes(code.to_ne_bytes()))
        .unwrap_or_default()
}

fn validate_size(cols: u16, rows: u16) -> Result<(), String> {
    if cols == 0 || rows == 0 || cols > i16::MAX as u16 || rows > i16::MAX as u16 {
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
