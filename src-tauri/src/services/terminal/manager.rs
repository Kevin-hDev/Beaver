use super::owned_session::OwnedSession;
use super::public_error::{map_admission_error, not_found, shutting_down, terminal_error};
use super::{generate_token, verify_token};
use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::ServiceWorkSupervisor;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::ipc::Channel;

static NEXT_ID: AtomicU32 = AtomicU32::new(1);
type TerminalWork = ServiceWorkSupervisor<16>;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyChannelEvent {
    pub data: String,
    pub is_exit: bool,
    pub exit_code: u32,
}

struct PtyState {
    closing: bool,
    sessions: HashMap<u32, OwnedSession>,
}

#[derive(Clone)]
pub struct PtyManager {
    state: Arc<Mutex<PtyState>>,
    work: TerminalWork,
}

impl PtyManager {
    pub const MAX_PTY_SESSIONS: usize = 16;

    pub fn new(app: AppWorkSupervisor) -> Self {
        Self {
            state: Arc::new(Mutex::new(PtyState {
                closing: false,
                sessions: HashMap::new(),
            })),
            work: TerminalWork::new(app),
        }
    }

    pub fn spawn(
        &self,
        on_output: Channel<PtyChannelEvent>,
        cwd: Option<&str>,
        cols: u16,
        rows: u16,
    ) -> Result<(u32, String), String> {
        self.spawn_with_sink(cwd, cols, rows, move |event| {
            let _ = on_output.send(event);
        })
    }

    fn spawn_with_sink(
        &self,
        cwd: Option<&str>,
        cols: u16,
        rows: u16,
        sink: impl Fn(PtyChannelEvent) + Send + 'static,
    ) -> Result<(u32, String), String> {
        self.reap_finished();
        let admission = self
            .work
            .try_admit()
            .map_err(|error| map_admission_error(error.public_code()))?;
        if self.is_closing()? {
            return Err(shutting_down());
        }
        let cancellation = admission.cancellation();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let token = generate_token();
        let token_copy = token.to_string();
        let mut owned = Some(OwnedSession::spawn(
            admission, token, cwd, cols, rows, sink,
        )?);
        if cancellation.is_cancelled() {
            owned.take().expect("owned PTY session").close();
            return Err(shutting_down());
        }
        let rejected = {
            let mut state = match self.lock_state() {
                Ok(state) => state,
                Err(error) => {
                    owned.take().expect("owned PTY session").close();
                    return Err(error);
                }
            };
            if state.closing
                || cancellation.is_cancelled()
                || state.sessions.len() >= Self::MAX_PTY_SESSIONS
                || state.sessions.contains_key(&id)
            {
                true
            } else {
                state
                    .sessions
                    .insert(id, owned.take().expect("owned PTY session"));
                false
            }
        };
        if rejected {
            owned.take().expect("rejected PTY session").close();
            return Err(shutting_down());
        }
        Ok((id, token_copy))
    }

    pub fn write(&self, id: u32, token: &str, data: &[u8]) -> Result<(), String> {
        self.with_session(id, token, |session| session.write(data))
    }

    pub fn resize(&self, id: u32, token: &str, cols: u16, rows: u16) -> Result<(), String> {
        self.with_session(id, token, |session| session.resize(cols, rows))
    }

    pub fn kill(&self, id: u32, token: &str) -> Result<(), String> {
        let owned = {
            let mut state = self.lock_state()?;
            let owned = state.sessions.get(&id).ok_or_else(not_found)?;
            verify_token(owned.token(), token)?;
            state.sessions.remove(&id).ok_or_else(not_found)?
        };
        owned.close();
        Ok(())
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.work.begin_closing();
        let sessions: Vec<OwnedSession> = {
            let Ok(mut state) = self.state.lock() else {
                return false;
            };
            state.closing = true;
            state.sessions.drain().map(|(_, owned)| owned).collect()
        };
        let close = tokio::task::spawn_blocking(move || {
            for owned in sessions {
                owned.close();
            }
        });
        if tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), close)
            .await
            .is_err()
        {
            return false;
        }
        self.work.stop_and_wait(deadline).await
    }

    fn with_session<Output>(
        &self,
        id: u32,
        token: &str,
        operation: impl FnOnce(&OwnedSession) -> Result<Output, String>,
    ) -> Result<Output, String> {
        let state = self.lock_state()?;
        let owned = state.sessions.get(&id).ok_or_else(not_found)?;
        verify_token(owned.token(), token)?;
        operation(owned)
    }

    fn reap_finished(&self) {
        let finished = {
            let Ok(mut state) = self.state.lock() else {
                return;
            };
            let ids = state
                .sessions
                .iter()
                .filter_map(|(id, owned)| owned.reader_finished().then_some(*id))
                .collect::<Vec<_>>();
            ids.into_iter()
                .filter_map(|id| state.sessions.remove(&id))
                .collect::<Vec<_>>()
        };
        for owned in finished {
            owned.close();
        }
    }

    fn is_closing(&self) -> Result<bool, String> {
        Ok(self.lock_state()?.closing)
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, PtyState>, String> {
        self.state.lock().map_err(|_| terminal_error())
    }

    #[cfg(test)]
    pub(crate) fn spawn_for_test(
        &self,
        cwd: Option<&str>,
        cols: u16,
        rows: u16,
    ) -> Result<(u32, String), String> {
        self.spawn_with_sink(cwd, cols, rows, |_| {})
    }

    #[cfg(test)]
    pub(crate) fn process_id_for_test(&self, id: u32) -> Option<u32> {
        self.state.lock().ok()?.sessions.get(&id)?.process_id()
    }

    #[cfg(test)]
    pub(crate) fn active_sessions_for_test(&self) -> usize {
        self.state
            .lock()
            .map(|state| state.sessions.len())
            .unwrap_or(Self::MAX_PTY_SESSIONS)
    }
}
