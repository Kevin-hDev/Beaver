use super::generate_token;
#[cfg(target_os = "linux")]
use super::linux_spawn_worker::LinuxSpawnWorker;
use super::owned_session::OwnedSession;
use super::public_error::{map_admission_error, not_found, shutting_down, terminal_error};
use super::session_handle::SessionHandle;
use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::ServiceWorkSupervisor;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::ipc::Channel;

pub(super) static NEXT_ID: AtomicU32 = AtomicU32::new(1);
type TerminalWork = ServiceWorkSupervisor<16>;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyChannelEvent {
    pub data: String,
    pub is_exit: bool,
    pub exit_code: u32,
}

pub(super) struct PtyState {
    pub(super) closing: bool,
    pub(super) sessions: HashMap<u32, Arc<SessionHandle>>,
}

#[derive(Clone)]
pub struct PtyManager {
    pub(super) state: Arc<Mutex<PtyState>>,
    work: TerminalWork,
    #[cfg(target_os = "linux")]
    linux_spawn: Arc<LinuxSpawnWorker>,
}

impl PtyManager {
    pub const MAX_PTY_SESSIONS: usize = 16;

    pub fn new(app: AppWorkSupervisor) -> Self {
        #[cfg(target_os = "linux")]
        let linux_spawn = Arc::new(LinuxSpawnWorker::new(ServiceWorkSupervisor::<1>::new(
            app.clone(),
        )));
        Self {
            state: Arc::new(Mutex::new(PtyState {
                closing: false,
                sessions: HashMap::new(),
            })),
            work: TerminalWork::new(app),
            #[cfg(target_os = "linux")]
            linux_spawn,
        }
    }

    pub fn spawn(
        &self,
        on_output: Channel<PtyChannelEvent>,
        cwd: Option<&Path>,
        cols: u16,
        rows: u16,
    ) -> Result<(u32, String), String> {
        self.spawn_with_sink(cwd, cols, rows, move |event| {
            on_output.send(event).map_err(|_| ())
        })
    }

    pub(super) fn spawn_with_sink(
        &self,
        cwd: Option<&Path>,
        cols: u16,
        rows: u16,
        sink: impl Fn(PtyChannelEvent) -> Result<(), ()> + Send + 'static,
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
        let (owned, control) = OwnedSession::spawn(admission, cwd, cols, rows, sink)?;
        let handle = Arc::new(SessionHandle::new(Box::new(owned), control, token));
        if cancellation.is_cancelled() {
            handle.close();
            return Err(shutting_down());
        }
        let rejected = {
            let mut state = self.lock_state()?;
            if state.closing
                || cancellation.is_cancelled()
                || state.sessions.len() >= Self::MAX_PTY_SESSIONS
                || state.sessions.contains_key(&id)
            {
                true
            } else {
                state.sessions.insert(id, Arc::clone(&handle));
                false
            }
        };
        if rejected {
            handle.close();
            return Err(shutting_down());
        }
        Ok((id, token_copy))
    }

    #[cfg(target_os = "linux")]
    pub async fn spawn_linux(
        &self,
        on_output: Channel<PtyChannelEvent>,
        cwd: std::path::PathBuf,
        cols: u16,
        rows: u16,
    ) -> Result<(u32, String), String> {
        let manager = self.clone();
        self.linux_spawn
            .submit(Box::new(move || {
                manager.spawn(on_output, Some(cwd.as_path()), cols, rows)
            }))
            .await
    }

    pub fn write(&self, id: u32, token: &str, data: &[u8]) -> Result<(), String> {
        self.session(id, token)?
            .with_live(|session| session.write(data))
    }

    pub fn resize(&self, id: u32, token: &str, cols: u16, rows: u16) -> Result<(), String> {
        self.session(id, token)?
            .with_live(|session| session.resize(cols, rows))
    }

    pub fn kill(&self, id: u32, token: &str) -> Result<(), String> {
        let handle = self.session(id, token)?;
        let removed = self
            .lock_state()?
            .sessions
            .remove(&id)
            .ok_or_else(not_found)?;
        debug_assert!(Arc::ptr_eq(&handle, &removed));
        removed.close();
        Ok(())
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.work.begin_closing();
        {
            let Ok(mut state) = self.state.lock() else {
                return false;
            };
            state.closing = true;
        }
        #[cfg(target_os = "linux")]
        {
            self.linux_spawn.begin_closing();
            if !self.linux_spawn.stop_and_wait(deadline).await {
                return false;
            }
        }
        let sessions: Vec<Arc<SessionHandle>> = {
            let Ok(mut state) = self.state.lock() else {
                return false;
            };
            state.sessions.drain().map(|(_, handle)| handle).collect()
        };
        let close = super::shutdown::run_until(deadline, move || {
            for handle in sessions {
                handle.close();
            }
        });
        close.await && self.work.stop_and_wait(deadline).await
    }

    fn session(&self, id: u32, token: &str) -> Result<Arc<SessionHandle>, String> {
        let handle = self
            .lock_state()?
            .sessions
            .get(&id)
            .cloned()
            .ok_or_else(not_found)?;
        handle.verify_token(token)?;
        Ok(handle)
    }

    fn reap_finished(&self) {
        let handles = match self.state.lock() {
            Ok(state) => state
                .sessions
                .iter()
                .map(|(id, handle)| (*id, Arc::clone(handle)))
                .collect::<Vec<_>>(),
            Err(_) => return,
        };
        let ids = handles
            .iter()
            .filter_map(|(id, handle)| handle.reader_finished().then_some(*id))
            .collect::<Vec<_>>();
        let finished = match self.state.lock() {
            Ok(mut state) => ids
                .into_iter()
                .filter_map(|id| state.sessions.remove(&id))
                .collect::<Vec<_>>(),
            Err(_) => return,
        };
        for handle in finished {
            handle.close();
        }
    }

    fn is_closing(&self) -> Result<bool, String> {
        Ok(self.lock_state()?.closing)
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, PtyState>, String> {
        self.state.lock().map_err(|_| terminal_error())
    }
}
