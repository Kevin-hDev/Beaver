use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

type SharedChild = Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>;

pub struct PtySession {
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    child: SharedChild,
    writer: Mutex<Option<Box<dyn Write + Send>>>,
}

const CHILD_REAP_TIMEOUT: Duration = Duration::from_secs(2);
const CHILD_REAP_POLL: Duration = Duration::from_millis(20);

#[derive(Clone)]
pub(crate) struct PtyChildStatus(SharedChild);

impl PtyChildStatus {
    pub(crate) fn exit_code(&self) -> Option<u32> {
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
        cwd: Option<&Path>,
        cols: u16,
        rows: u16,
    ) -> Result<(Self, Box<dyn Read + Send>), String> {
        validate_size(cols, rows)?;
        let pair = NativePtySystem::default()
            .openpty(pty_size(cols, rows))
            .map_err(|_| terminal_error())?;
        let shell = default_shell()?;
        let mut command = CommandBuilder::new(&shell);
        command.arg("-l");
        command.env("TERM", "xterm-256color");
        if std::env::var("EDITOR").is_ok_and(|editor| editor.contains("vi")) {
            command.env("EDITOR", "");
        }
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
        reap_within(&mut **child, CHILD_REAP_TIMEOUT).ok_or_else(terminal_error)
    }

    fn close_input(&self) -> Result<(), String> {
        let writer = self.writer.lock().map_err(|_| terminal_error())?.take();
        drop(writer);
        Ok(())
    }
}

/// Récolte l'enfant sans jamais dépasser `timeout`, et rend `None` s'il ne
/// meurt pas dans ce budget. Un `Drop` s'exécute sur n'importe quel fil, y
/// compris celui qui ferme l'application : une attente sans borne y fige tout
/// ce qui suit. Passé le délai le processus reste zombie jusqu'à la sortie de
/// Beaver, ce que le système récupère seul.
fn reap_within(
    child: &mut (dyn portable_pty::Child + Send + Sync),
    timeout: Duration,
) -> Option<u32> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status.exit_code()),
            Ok(None) => {}
            Err(_) => return None,
        }
        if Instant::now() >= deadline {
            ::log::warn!("[terminal] shell non récolté sous {timeout:?}, abandon de l'attente");
            return None;
        }
        std::thread::sleep(CHILD_REAP_POLL);
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
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let path = std::path::Path::new(&shell);
    if !path.is_absolute() || !path.is_file() {
        return Err("terminal-shell-invalid".to_string());
    }
    Ok(shell)
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
