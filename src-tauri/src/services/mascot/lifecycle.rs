use super::event_mapping::{FAILURE_DURATION, SUCCESS_DURATION};
use super::{update_activity, MascotAnimation};
use std::time::Instant;
use tauri::AppHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MascotSessionOutcome {
    Success,
    Failed,
    Cancelled,
}

pub struct MascotSession {
    app: AppHandle,
    session_id: String,
    generation: u64,
    active: bool,
}

impl MascotSession {
    pub fn start(app: &AppHandle, session_id: String, generation: u64) -> Self {
        update_activity(app, |arbiter| {
            arbiter.start(&session_id, generation, Instant::now())
        });
        Self {
            app: app.clone(),
            session_id,
            generation,
            active: true,
        }
    }

    pub fn finish(mut self, outcome: MascotSessionOutcome) {
        match outcome {
            MascotSessionOutcome::Success => update_activity(&self.app, |arbiter| {
                arbiter.update(
                    &self.session_id,
                    Some(self.generation),
                    MascotAnimation::Success,
                    Some(SUCCESS_DURATION),
                    false,
                    Instant::now(),
                )
            }),
            MascotSessionOutcome::Failed => update_activity(&self.app, |arbiter| {
                arbiter.update(
                    &self.session_id,
                    Some(self.generation),
                    MascotAnimation::Failed,
                    Some(FAILURE_DURATION),
                    false,
                    Instant::now(),
                )
            }),
            MascotSessionOutcome::Cancelled => {
                cancel_session(&self.app, &self.session_id, self.generation);
            }
        }
        self.active = false;
    }
}

impl Drop for MascotSession {
    fn drop(&mut self) {
        if self.active {
            cancel_session(&self.app, &self.session_id, self.generation);
        }
    }
}

pub fn cancel_session(app: &AppHandle, session_id: &str, generation: u64) {
    update_activity(app, |arbiter| {
        arbiter.remove(session_id, generation, Instant::now())
    });
}
