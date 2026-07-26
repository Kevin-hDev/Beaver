use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

pub struct PtySession {
    master: Box<dyn portable_pty::MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl PtySession {
    pub fn spawn(
        cwd: Option<&str>,
        cols: u16,
        rows: u16,
    ) -> Result<(Self, Box<dyn Read + Send>), String> {
        let pty_system = NativePtySystem::default();

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system
            .openpty(size)
            .map_err(|e| format!("openpty failed: {}", e))?;

        let shell = default_shell()?;
        let mut cmd = CommandBuilder::new(&shell);

        #[cfg(unix)]
        cmd.arg("-l");

        cmd.env("TERM", "xterm-256color");
        // Empêche zsh de basculer en mode vi si EDITOR contient "vi"
        if let Ok(editor) = std::env::var("EDITOR") {
            if editor.contains("vi") {
                cmd.env("EDITOR", "");
            }
        }

        if let Some(dir) = cwd {
            let path = std::path::Path::new(dir);
            if !path.is_absolute() || !path.is_dir() {
                return Err("Invalid working directory".to_string());
            }
            cmd.cwd(dir);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("spawn failed: {}", e))?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("clone reader: {}", e))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("take writer: {}", e))?;

        let session = Self {
            master: pair.master,
            child,
            writer: Arc::new(Mutex::new(writer)),
        };

        Ok((session, reader))
    }

    const MAX_WRITE_BYTES: usize = 65_536;

    pub fn write(&self, data: &[u8]) -> Result<(), String> {
        if data.len() > Self::MAX_WRITE_BYTES {
            return Err("Write payload too large".to_string());
        }
        let mut w = self.writer.lock().map_err(|e| format!("lock: {}", e))?;
        w.write_all(data).map_err(|e| format!("write: {}", e))?;
        w.flush().map_err(|e| format!("flush: {}", e))?;
        Ok(())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("resize: {}", e))
    }

    pub fn kill(&mut self) -> Result<(), String> {
        self.child.kill().map_err(|e| format!("kill: {}", e))
    }

    pub fn try_wait(&mut self) -> Option<u32> {
        self.child
            .try_wait()
            .ok()
            .flatten()
            .map(|status| status.exit_code())
    }
}

fn default_shell() -> Result<String, String> {
    #[cfg(unix)]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let path = std::path::Path::new(&shell);
        if !path.is_absolute() || !path.is_file() {
            return Err("Invalid shell binary".to_string());
        }
        Ok(shell)
    }
    #[cfg(windows)]
    {
        Ok("powershell.exe".to_string())
    }
}
