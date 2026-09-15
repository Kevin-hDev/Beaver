use std::{
    path::Path,
    sync::mpsc::{Receiver, TryRecvError},
};

use sherpa_onnx::VoiceActivityDetector;

use crate::services::{
    voice::{download::VoiceCatalogEntry, errors::VoiceError, types::VoiceUnloadDelay},
    work_registry::ServiceWorkAdmissionError,
};

use super::{
    lifecycle::{receipt, ModelLease, ModelLifecycle},
    lifecycle_loading_worker::{spawn, LoadRequest},
    lifecycle_release::release_model,
    lifecycle_state::ModelKey,
    recognizer::{ExecutionProfile, PreparedModel},
};

pub enum CaptureModelLease {
    Ready(Box<ModelLease>),
    Loading(Box<LoadingModelLease>),
}

pub struct LoadingModelLease {
    lifecycle: ModelLifecycle,
    key: ModelKey,
    delay: VoiceUnloadDelay,
    vad: VoiceActivityDetector,
    result: Option<Receiver<Result<PreparedModel, VoiceError>>>,
    ready: Option<ModelLease>,
    finished: bool,
}

impl ModelLifecycle {
    pub fn acquire_for_capture(
        &self,
        data_dir: &Path,
        asr: &VoiceCatalogEntry,
        vad: &VoiceCatalogEntry,
        profile: ExecutionProfile,
        delay: VoiceUnloadDelay,
    ) -> Result<CaptureModelLease, VoiceError> {
        profile.validate()?;
        let key = ModelKey::new(asr, vad, profile.clone());
        let previous = self.begin_acquire(&key)?;
        let asr_receipt = receipt(asr, data_dir).inspect_err(|_| self.fail_acquire(&key))?;
        let vad_receipt = receipt(vad, data_dir).inspect_err(|_| self.fail_acquire(&key))?;
        if let Some(loaded) = previous {
            if loaded.key == key {
                return Ok(CaptureModelLease::Ready(Box::new(ModelLease {
                    inner: self.inner.clone(),
                    model: Some(loaded.model),
                    key,
                    delay,
                })));
            }
            drop(loaded);
        }
        self.verify_once(data_dir, [&vad_receipt])
            .inspect_err(|_| self.fail_acquire(&key))?;
        let capture_vad = super::vad::load(&vad_receipt.install_dir(data_dir), &profile)
            .ok_or_else(VoiceError::configuration_unavailable)
            .inspect_err(|_| self.fail_acquire(&key))?;
        let admission = self
            .work
            .try_admit()
            .map_err(map_admission_error)
            .inspect_err(|_| self.fail_acquire(&key))?;
        let result = spawn(
            LoadRequest {
                lifecycle: self.clone(),
                key: key.clone(),
                delay,
                data_dir: data_dir.to_path_buf(),
                asr: asr.clone(),
                vad: vad.clone(),
                asr_receipt,
                vad_receipt,
                profile,
            },
            admission,
        )
        .inspect_err(|_| self.fail_acquire(&key))?;
        Ok(CaptureModelLease::Loading(Box::new(LoadingModelLease {
            lifecycle: self.clone(),
            key,
            delay,
            vad: capture_vad,
            result: Some(result),
            ready: None,
            finished: false,
        })))
    }
}

impl CaptureModelLease {
    pub fn vad(&self) -> &VoiceActivityDetector {
        match self {
            Self::Ready(lease) => lease.prepared().vad(),
            Self::Loading(loading) => loading.vad(),
        }
    }

    pub fn poll_loading(&mut self) -> Result<(), VoiceError> {
        if let Self::Loading(loading) = self {
            loading.poll()?;
        }
        Ok(())
    }

    pub fn into_ready(self) -> Result<ModelLease, VoiceError> {
        match self {
            Self::Ready(lease) => Ok(*lease),
            Self::Loading(loading) => (*loading).finish(),
        }
    }
}

impl LoadingModelLease {
    fn vad(&self) -> &VoiceActivityDetector {
        &self.vad
    }

    fn poll(&mut self) -> Result<(), VoiceError> {
        if self.ready.is_some() {
            return Ok(());
        }
        let received = match self
            .result
            .as_ref()
            .expect("loading result remains open")
            .try_recv()
        {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return Ok(()),
            Err(TryRecvError::Disconnected) => {
                self.lifecycle.fail_acquire(&self.key);
                self.finished = true;
                return Err(VoiceError::configuration_unavailable());
            }
        };
        self.ready = Some(self.accept(received)?);
        Ok(())
    }

    fn finish(mut self) -> Result<ModelLease, VoiceError> {
        if let Some(ready) = self.ready.take() {
            return Ok(ready);
        }
        let received = self
            .result
            .as_ref()
            .expect("loading result remains open")
            .recv()
            .map_err(|_| VoiceError::configuration_unavailable())?;
        self.accept(received)
    }

    fn accept(
        &mut self,
        received: Result<PreparedModel, VoiceError>,
    ) -> Result<ModelLease, VoiceError> {
        self.result.take();
        self.finished = true;
        let model = received?;
        Ok(ModelLease {
            inner: self.lifecycle.inner.clone(),
            model: Some(model),
            key: self.key.clone(),
            delay: self.delay,
        })
    }
}

impl Drop for LoadingModelLease {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        match self.result.take().and_then(|result| result.try_recv().ok()) {
            Some(Ok(model)) => release_model(&self.lifecycle.inner, &self.key, self.delay, model),
            Some(Err(_)) => self.lifecycle.fail_acquire(&self.key),
            None => {}
        }
    }
}

fn map_admission_error(error: ServiceWorkAdmissionError) -> VoiceError {
    match error {
        ServiceWorkAdmissionError::AppClosing | ServiceWorkAdmissionError::Closing => {
            VoiceError::shutting_down()
        }
        ServiceWorkAdmissionError::AppCapacity | ServiceWorkAdmissionError::Capacity => {
            VoiceError::busy()
        }
    }
}
