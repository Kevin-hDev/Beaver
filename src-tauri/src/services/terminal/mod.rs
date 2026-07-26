pub mod pty_session;

#[cfg(test)]
mod tests;

use pty_session::PtySession;
use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

pub struct PtyManager {
    sessions: Arc<Mutex<HashMap<u32, OwnedSession>>>,
}

struct OwnedSession {
    session: PtySession,
    token: zeroize::Zeroizing<String>,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyChannelEvent {
    pub data: String,
    pub is_exit: bool,
    pub exit_code: u32,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    const MAX_PTY_SESSIONS: usize = 16;

    pub fn spawn(
        &self,
        on_output: Channel<PtyChannelEvent>,
        cwd: Option<&str>,
        cols: u16,
        rows: u16,
    ) -> Result<(u32, String), String> {
        {
            let sessions = self.sessions.lock().map_err(|e| format!("lock: {e}"))?;
            if sessions.len() >= Self::MAX_PTY_SESSIONS {
                return Err("Trop de terminaux ouverts".to_string());
            }
        }
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let token = generate_token();
        let (session, reader) = PtySession::spawn(cwd, cols, rows)?;

        let token_copy = token.to_string();
        self.sessions
            .lock()
            .map_err(|e| format!("lock: {}", e))?
            .insert(id, OwnedSession { session, token });

        self.start_reader_thread(id, reader, on_output);

        Ok((id, token_copy))
    }

    pub fn write(&self, id: u32, token: &str, data: &[u8]) -> Result<(), String> {
        let sessions = self.sessions.lock().map_err(|e| format!("lock: {}", e))?;
        let owned = sessions
            .get(&id)
            .ok_or_else(|| "terminal introuvable".to_string())?;
        verify_token(&owned.token, token)?;
        owned.session.write(data)
    }

    pub fn resize(&self, id: u32, token: &str, cols: u16, rows: u16) -> Result<(), String> {
        let sessions = self.sessions.lock().map_err(|e| format!("lock: {}", e))?;
        let owned = sessions
            .get(&id)
            .ok_or_else(|| "terminal introuvable".to_string())?;
        verify_token(&owned.token, token)?;
        owned.session.resize(cols, rows)
    }

    pub fn kill(&self, id: u32, token: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| format!("lock: {}", e))?;
        let owned = sessions
            .get(&id)
            .ok_or_else(|| "terminal introuvable".to_string())?;
        verify_token(&owned.token, token)?;
        let mut owned = sessions.remove(&id).unwrap();
        owned.session.kill()
    }

    pub fn kill_all(&self) {
        if let Ok(mut sessions) = self.sessions.lock() {
            for (_, mut owned) in sessions.drain() {
                let _ = owned.session.kill();
            }
        }
    }

    fn start_reader_thread(
        &self,
        id: u32,
        mut reader: Box<dyn Read + Send>,
        channel: Channel<PtyChannelEvent>,
    ) {
        let sessions = Arc::clone(&self.sessions);

        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = channel.send(PtyChannelEvent {
                            data: text,
                            is_exit: false,
                            exit_code: 0,
                        });
                    }
                    Err(_) => break,
                }
            }

            let code = sessions
                .lock()
                .ok()
                .and_then(|mut s| s.remove(&id))
                .and_then(|mut owned| owned.session.try_wait())
                .unwrap_or(0);

            let _ = channel.send(PtyChannelEvent {
                data: String::new(),
                is_exit: true,
                exit_code: code,
            });
        });
    }
}

fn generate_token() -> zeroize::Zeroizing<String> {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let s = bytes.iter().map(|b| format!("{b:02x}")).collect();
    bytes.fill(0);
    zeroize::Zeroizing::new(s)
}

fn verify_token(expected: &str, provided: &str) -> Result<(), String> {
    if expected.len() != provided.len() {
        return Err("accès terminal refusé".to_string());
    }
    let mismatch = expected
        .as_bytes()
        .iter()
        .zip(provided.as_bytes().iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b));
    if mismatch != 0 {
        return Err("accès terminal refusé".to_string());
    }
    Ok(())
}
