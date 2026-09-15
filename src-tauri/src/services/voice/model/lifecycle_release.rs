use std::sync::Arc;

use crate::services::voice::types::VoiceUnloadDelay;

use super::{
    lifecycle::ModelLease,
    lifecycle_state::{lock, LifecycleInner, LoadedModel, ModelKey, UnloadPlan},
    recognizer::PreparedModel,
};

pub(super) fn release_model(
    inner: &Arc<LifecycleInner>,
    key: &ModelKey,
    delay: VoiceUnloadDelay,
    model: PreparedModel,
) {
    let mut state = lock(&inner.state);
    if state.occupied.as_ref() != Some(key) {
        return;
    }
    state.occupied = None;
    state.generation = state.generation.wrapping_add(1);
    let generation = state.generation;
    if state.closing || delay == VoiceUnloadDelay::Immediately {
        return;
    }
    state.loaded = Some(LoadedModel {
        key: key.clone(),
        model,
    });
    if inner
        .plan
        .send(UnloadPlan {
            generation,
            deadline: super::lifecycle_maintenance::deadline(delay),
        })
        .is_err()
    {
        drop(state.loaded.take());
    }
}

impl Drop for ModelLease {
    fn drop(&mut self) {
        if let Some(model) = self.model.take() {
            release_model(&self.inner, &self.key, self.delay, model);
        }
    }
}
