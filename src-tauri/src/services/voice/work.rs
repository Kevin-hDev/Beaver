use super::errors::{VoiceError, VoiceErrorCode};
use super::state::VoiceState;
use super::types::VoicePhase;
use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::{
    ServiceWorkAdmission, ServiceWorkAdmissionError, ServiceWorkCancellation, ServiceWorkSupervisor,
};
use std::sync::{Arc, Mutex, MutexGuard};

type VoiceSupervisor = ServiceWorkSupervisor<{ super::limits::MAX_VOICE_OPERATIONS }>;

#[derive(Clone)]
pub(super) struct VoiceWork {
    supervisor: VoiceSupervisor,
    state: Arc<Mutex<VoiceState>>,
}

pub struct VoiceWorkContext {
    cancellation: ServiceWorkCancellation,
    state: Arc<Mutex<VoiceState>>,
}

pub(super) struct VoiceOwner {
    admission: Option<ServiceWorkAdmission<{ super::limits::MAX_VOICE_OPERATIONS }>>,
    state: Arc<Mutex<VoiceState>>,
}

impl VoiceWork {
    pub(super) fn new(app: AppWorkSupervisor) -> Self {
        Self {
            supervisor: VoiceSupervisor::new(app),
            state: Arc::new(Mutex::new(VoiceState::default())),
        }
    }

    pub(super) fn try_reserve(&self) -> Result<VoiceOwner, VoiceError> {
        let mut state = lock_state(&self.state);
        state.begin()?;
        let admission = match self.supervisor.try_admit() {
            Ok(admission) => admission,
            Err(error) => {
                state.finish();
                return Err(map_admission_error(error));
            }
        };
        drop(state);
        Ok(VoiceOwner {
            admission: Some(admission),
            state: Arc::clone(&self.state),
        })
    }

    pub(super) fn phase(&self) -> VoicePhase {
        lock_state(&self.state).phase()
    }

    pub(super) fn active(&self) -> usize {
        self.supervisor.diagnostics().active
    }

    pub(super) fn begin_closing(&self) {
        self.supervisor.begin_closing();
    }

    pub(super) async fn stop_and_wait(&self, deadline: std::time::Instant) -> bool {
        self.supervisor.stop_and_wait(deadline).await
    }
}

impl VoiceOwner {
    pub(super) fn context(&self) -> VoiceWorkContext {
        VoiceWorkContext {
            cancellation: self
                .admission
                .as_ref()
                .expect("voice owner retains its admission")
                .cancellation(),
            state: Arc::clone(&self.state),
        }
    }
}

impl Drop for VoiceOwner {
    fn drop(&mut self) {
        drop(self.admission.take());
        lock_state(&self.state).finish();
    }
}

impl VoiceWorkContext {
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    pub fn transition(&self, phase: VoicePhase) -> Result<(), VoiceError> {
        lock_state(&self.state).transition(phase)
    }
}

fn lock_state(state: &Mutex<VoiceState>) -> MutexGuard<'_, VoiceState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn map_admission_error(error: ServiceWorkAdmissionError) -> VoiceError {
    let code = match error {
        ServiceWorkAdmissionError::AppClosing | ServiceWorkAdmissionError::Closing => {
            VoiceErrorCode::ShuttingDown
        }
        ServiceWorkAdmissionError::AppCapacity | ServiceWorkAdmissionError::Capacity => {
            VoiceErrorCode::Busy
        }
    };
    VoiceError::from_code(code)
}
