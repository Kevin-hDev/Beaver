use std::{path::PathBuf, sync::mpsc};

use crate::services::{
    voice::{
        download::{InstallationReceipt, VoiceCatalogEntry},
        errors::VoiceError,
        types::VoiceUnloadDelay,
    },
    work_registry::ServiceWorkAdmission,
};

use super::{
    lifecycle::ModelLifecycle,
    lifecycle_release::release_model,
    lifecycle_state::ModelKey,
    recognizer::{ExecutionProfile, PreparedModel},
};

pub(super) struct LoadRequest {
    pub(super) lifecycle: ModelLifecycle,
    pub(super) key: ModelKey,
    pub(super) delay: VoiceUnloadDelay,
    pub(super) data_dir: PathBuf,
    pub(super) asr: VoiceCatalogEntry,
    pub(super) vad: VoiceCatalogEntry,
    pub(super) asr_receipt: InstallationReceipt,
    pub(super) vad_receipt: InstallationReceipt,
    pub(super) profile: ExecutionProfile,
}

pub(super) fn spawn(
    request: LoadRequest,
    admission: ServiceWorkAdmission<2>,
) -> Result<mpsc::Receiver<Result<PreparedModel, VoiceError>>, VoiceError> {
    // Canal sans tampon : si l'écoute est annulée, le worker récupère encore
    // le modèle et peut le rendre au cache au lieu de perdre sa réservation.
    let (sender, receiver) = mpsc::sync_channel(0);
    let cancellation = admission.cancellation();
    std::thread::Builder::new()
        .name("voice-asr-loader".into())
        .spawn(move || {
            let _admission = admission;
            let mut guard = ReservationGuard::new(request.lifecycle.clone(), request.key.clone());
            let loaded = if cancellation.is_cancelled() {
                Err(VoiceError::shutting_down())
            } else {
                request
                    .lifecycle
                    .verify_once(
                        &request.data_dir,
                        [&request.asr_receipt, &request.vad_receipt],
                    )
                    .and_then(|()| {
                        PreparedModel::load(
                            &request.asr,
                            &request.asr_receipt,
                            &request.vad_receipt,
                            &request.data_dir,
                            request.profile,
                        )
                    })
            };
            match loaded {
                Ok(model) => match sender.send(Ok(model)) {
                    Ok(()) => guard.disarm(),
                    Err(error) => {
                        if let Ok(model) = error.0 {
                            release_model(
                                &request.lifecycle.inner,
                                &request.key,
                                request.delay,
                                model,
                            );
                        }
                        guard.disarm();
                    }
                },
                Err(error) => {
                    guard.fail_now();
                    let _ = sender.send(Err(error));
                }
            }
        })
        .map_err(|_| VoiceError::configuration_unavailable())?;
    Ok(receiver)
}

struct ReservationGuard {
    lifecycle: ModelLifecycle,
    key: ModelKey,
    armed: bool,
}

impl ReservationGuard {
    fn new(lifecycle: ModelLifecycle, key: ModelKey) -> Self {
        Self {
            lifecycle,
            key,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }

    fn fail_now(&mut self) {
        self.lifecycle.fail_acquire(&self.key);
        self.disarm();
    }
}

impl Drop for ReservationGuard {
    fn drop(&mut self) {
        if self.armed {
            self.lifecycle.fail_acquire(&self.key);
        }
    }
}
