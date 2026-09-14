use super::voice_probe_audio::{build_input_stream, InputUpdate};
use super::voice_probe_state::{ProbeState, VoiceProbeError};
use crate::app_exit::AppWorkSupervisor;
use crate::services::work_registry::{ServiceWorkCancellation, ServiceWorkSupervisor};
use cpal::traits::StreamTrait;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[cfg(target_os = "macos")]
use block2::RcBlock;
#[cfg(target_os = "macos")]
use objc2::runtime::Bool;
#[cfg(target_os = "macos")]
use objc2_av_foundation::{AVAuthorizationStatus, AVCaptureDevice, AVMediaTypeAudio};

pub use super::voice_probe_state::VoiceProbeSnapshot;
#[cfg(test)]
pub(super) use super::voice_probe_state::{
    ProbeState as TestedProbeState, VoiceProbeStatus, MAX_LEVEL_HISTORY,
};

const PROBE_DURATION: Duration = Duration::from_secs(5);
const LEVEL_INTERVAL: Duration = Duration::from_millis(50);

pub struct VoiceProbeRuntime {
    work: ServiceWorkSupervisor<1>,
    state: Arc<Mutex<ProbeState>>,
    cancellation: Arc<Mutex<Option<ServiceWorkCancellation>>>,
}

impl VoiceProbeRuntime {
    pub fn new(app_work: AppWorkSupervisor) -> Self {
        Self {
            work: ServiceWorkSupervisor::new(app_work),
            state: Arc::new(Mutex::new(ProbeState::default())),
            cancellation: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&self, app: AppHandle) -> Result<VoiceProbeSnapshot, VoiceProbeError> {
        self.lock_state()?.begin()?;
        let admission = self.work.try_admit().map_err(|_| self.fail())?;
        *self.lock_cancellation()? = Some(admission.cancellation());
        let initial = self.snapshot()?;
        emit_snapshot(&app, &initial);

        let state = Arc::clone(&self.state);
        let active_cancellation = Arc::clone(&self.cancellation);
        if admission
            .spawn(move |cancel| run_probe(cancel, state, active_cancellation, app))
            .is_err()
        {
            self.lock_cancellation()?.take();
            return Err(self.fail());
        }
        Ok(initial)
    }

    pub fn stop(&self) -> Result<VoiceProbeSnapshot, VoiceProbeError> {
        if let Some(cancellation) = self.lock_cancellation()?.take() {
            cancellation.cancel();
            self.lock_state()?.mark_stopping();
        } else {
            self.lock_state()?.finish();
        }
        self.snapshot()
    }

    fn snapshot(&self) -> Result<VoiceProbeSnapshot, VoiceProbeError> {
        Ok(self.lock_state()?.snapshot())
    }

    fn fail(&self) -> VoiceProbeError {
        if let Ok(mut state) = self.state.lock() {
            state.fail();
        }
        VoiceProbeError::Unavailable
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, ProbeState>, VoiceProbeError> {
        self.state.lock().map_err(|_| VoiceProbeError::Unavailable)
    }

    fn lock_cancellation(
        &self,
    ) -> Result<MutexGuard<'_, Option<ServiceWorkCancellation>>, VoiceProbeError> {
        self.cancellation
            .lock()
            .map_err(|_| VoiceProbeError::Unavailable)
    }
}

async fn run_probe(
    cancellation: ServiceWorkCancellation,
    state: Arc<Mutex<ProbeState>>,
    active_cancellation: Arc<Mutex<Option<ServiceWorkCancellation>>>,
    app: AppHandle,
) {
    let permission = tokio::select! {
        _ = cancellation.cancelled() => None,
        result = request_microphone_permission() => Some(result),
    };
    if !matches!(permission, Some(Ok(()))) {
        finish_probe(&state, &active_cancellation, &app, permission.is_some());
        return;
    }

    let (stream, updates) = match build_input_stream() {
        Ok(pair) => pair,
        Err(_) => {
            finish_probe(&state, &active_cancellation, &app, true);
            return;
        }
    };
    if stream.play().is_err() {
        finish_probe(&state, &active_cancellation, &app, true);
        return;
    }
    if let Ok(mut current) = state.lock() {
        current.mark_listening();
        emit_snapshot(&app, &current.snapshot());
    }

    let deadline = tokio::time::sleep(PROBE_DURATION);
    tokio::pin!(deadline);
    let mut interval = tokio::time::interval(LEVEL_INTERVAL);
    let mut sample_count = 0_u64;
    let mut failed = false;
    loop {
        tokio::select! {
            _ = cancellation.cancelled() => break,
            _ = &mut deadline => break,
            _ = interval.tick() => {
                while let Ok(update) = updates.try_recv() {
                    match update {
                        InputUpdate::Samples { count, level } => {
                            sample_count = sample_count.saturating_add(count);
                            if let Ok(mut current) = state.lock() {
                                current.record(sample_count, level);
                                emit_snapshot(&app, &current.snapshot());
                            }
                        }
                        InputUpdate::Failed => failed = true,
                    }
                }
                if failed { break; }
            }
        }
    }
    drop(stream);
    finish_probe(&state, &active_cancellation, &app, failed);
}

fn finish_probe(
    state: &Mutex<ProbeState>,
    active_cancellation: &Mutex<Option<ServiceWorkCancellation>>,
    app: &AppHandle,
    failed: bool,
) {
    if let Ok(mut token) = active_cancellation.lock() {
        token.take();
    }
    if let Ok(mut current) = state.lock() {
        if failed {
            current.fail();
        } else {
            current.finish();
        }
        emit_snapshot(app, &current.snapshot());
    }
}

#[cfg(target_os = "macos")]
async fn request_microphone_permission() -> Result<(), VoiceProbeError> {
    let media_type = unsafe { AVMediaTypeAudio }.ok_or(VoiceProbeError::Unavailable)?;
    let status = unsafe { AVCaptureDevice::authorizationStatusForMediaType(media_type) };
    if status == AVAuthorizationStatus::Authorized {
        return Ok(());
    }
    if status != AVAuthorizationStatus::NotDetermined {
        return Err(VoiceProbeError::Unavailable);
    }

    let (sender, receiver) = tokio::sync::oneshot::channel();
    let sender = Mutex::new(Some(sender));
    {
        let completion = RcBlock::new(move |granted: Bool| {
            if let Ok(mut sender) = sender.lock() {
                if let Some(sender) = sender.take() {
                    let _ = sender.send(granted.as_bool());
                }
            }
        });
        unsafe {
            AVCaptureDevice::requestAccessForMediaType_completionHandler(media_type, &completion);
        }
    }
    match receiver.await {
        Ok(true) => Ok(()),
        Ok(false) | Err(_) => Err(VoiceProbeError::Unavailable),
    }
}

#[cfg(windows)]
async fn request_microphone_permission() -> Result<(), VoiceProbeError> {
    Ok(())
}

fn emit_snapshot(app: &AppHandle, snapshot: &VoiceProbeSnapshot) {
    if app.emit("voice-probe-snapshot", snapshot).is_err() {
        ::log::warn!("voice_probe_snapshot_delivery_failed");
    }
}
