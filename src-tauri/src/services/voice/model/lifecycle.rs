use std::{path::Path, sync::Arc};

use crate::services::voice::{
    download::{
        installed_receipt, verify_file, InstallationReceipt, RemovalGate, VoiceCatalogEntry,
    },
    errors::VoiceError,
    types::VoiceUnloadDelay,
};

use super::{
    lifecycle_state::{lock, LifecycleInner, LoadedModel, ModelKey, UnloadPlan},
    recognizer::{ExecutionProfile, PreparedModel},
};

#[derive(Clone)]
pub struct ModelLifecycle {
    pub(super) inner: Arc<LifecycleInner>,
    pub(super) maintenance: crate::services::work_registry::ServiceWorkSupervisor<1>,
}

pub struct ModelLease {
    inner: Arc<LifecycleInner>,
    model: Option<PreparedModel>,
    key: ModelKey,
    delay: VoiceUnloadDelay,
}

impl ModelLifecycle {
    pub fn acquire(
        &self,
        data_dir: &Path,
        asr: &VoiceCatalogEntry,
        vad: &VoiceCatalogEntry,
        profile: ExecutionProfile,
        delay: VoiceUnloadDelay,
    ) -> Result<ModelLease, VoiceError> {
        profile.validate()?;
        let key = ModelKey::new(asr, vad, profile.clone());
        let previous = self.begin_acquire(&key)?;
        let asr_receipt = match receipt(asr, data_dir) {
            Ok(receipt) => receipt,
            Err(error) => {
                self.fail_acquire(&key);
                return Err(error);
            }
        };
        let vad_receipt = match receipt(vad, data_dir) {
            Ok(receipt) => receipt,
            Err(error) => {
                self.fail_acquire(&key);
                return Err(error);
            }
        };
        if let Some(loaded) = previous {
            if loaded.key == key {
                return Ok(ModelLease {
                    inner: Arc::clone(&self.inner),
                    model: Some(loaded.model),
                    key,
                    delay,
                });
            }
            drop(loaded);
        }
        if let Err(error) = self.verify_once(data_dir, [&asr_receipt, &vad_receipt]) {
            self.fail_acquire(&key);
            return Err(error);
        }
        let model = PreparedModel::load(asr, &asr_receipt, &vad_receipt, data_dir, profile)
            .inspect_err(|_| self.fail_acquire(&key))?;
        Ok(ModelLease {
            inner: Arc::clone(&self.inner),
            model: Some(model),
            key,
            delay,
        })
    }

    pub fn invalidate_installation(&self, model_id: &str) -> Result<(), VoiceError> {
        let mut state = lock(&self.inner.state);
        if state
            .occupied
            .as_ref()
            .is_some_and(|key| key.uses(model_id))
        {
            return Err(VoiceError::busy());
        }
        state.verified.retain(|(id, _)| id != model_id);
        if state
            .loaded
            .as_ref()
            .is_some_and(|loaded| loaded.key.uses(model_id))
        {
            drop(state.loaded.take());
        }
        Ok(())
    }

    pub(super) fn begin_acquire(&self, key: &ModelKey) -> Result<Option<LoadedModel>, VoiceError> {
        let mut state = lock(&self.inner.state);
        if state.closing {
            return Err(VoiceError::shutting_down());
        }
        if state.occupied.is_some() {
            return Err(VoiceError::busy());
        }
        state.generation = state.generation.wrapping_add(1);
        state.occupied = Some(key.clone());
        if self
            .inner
            .plan
            .send(UnloadPlan {
                generation: state.generation,
                deadline: None,
            })
            .is_err()
        {
            state.occupied = None;
            return Err(VoiceError::shutting_down());
        }
        Ok(state.loaded.take())
    }

    fn fail_acquire(&self, key: &ModelKey) {
        let mut state = lock(&self.inner.state);
        if state.occupied.as_ref() == Some(key) {
            state.occupied = None;
        }
    }

    pub(super) fn verify_once(
        &self,
        data_dir: &Path,
        receipts: [&InstallationReceipt; 2],
    ) -> Result<(), VoiceError> {
        for receipt in receipts {
            let fingerprint = (receipt.entry_id.clone(), receipt.revision.clone());
            if lock(&self.inner.state).verified.contains(&fingerprint) {
                continue;
            }
            let root = receipt.install_dir(data_dir);
            for file in &receipt.files {
                verify_file(&root.join(&file.path), file.bytes, &file.sha256)
                    .map_err(|_| VoiceError::configuration_unavailable())?;
            }
            lock(&self.inner.state).verified.insert(fingerprint);
        }
        Ok(())
    }
}

impl ModelLease {
    pub fn prepared_mut(&mut self) -> &mut PreparedModel {
        self.model.as_mut().expect("model lease owns the model")
    }
}

impl Drop for ModelLease {
    fn drop(&mut self) {
        let Some(model) = self.model.take() else {
            return;
        };
        let mut state = lock(&self.inner.state);
        state.occupied = None;
        state.generation = state.generation.wrapping_add(1);
        let generation = state.generation;
        if state.closing || self.delay == VoiceUnloadDelay::Immediately {
            drop(state);
            drop(model);
            return;
        }
        state.loaded = Some(LoadedModel {
            key: self.key.clone(),
            model,
        });
        if self
            .inner
            .plan
            .send(UnloadPlan {
                generation,
                deadline: super::lifecycle_maintenance::deadline(self.delay),
            })
            .is_err()
        {
            drop(state.loaded.take());
        }
    }
}

impl RemovalGate for ModelLifecycle {
    fn release_for_removal(&self, model_id: &str) -> bool {
        self.invalidate_installation(model_id).is_ok()
    }
}

fn receipt(entry: &VoiceCatalogEntry, data_dir: &Path) -> Result<InstallationReceipt, VoiceError> {
    installed_receipt(entry, data_dir)
        .map_err(|_| VoiceError::configuration_unavailable())?
        .ok_or_else(VoiceError::configuration_unavailable)
}
