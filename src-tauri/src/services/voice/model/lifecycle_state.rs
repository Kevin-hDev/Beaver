use std::{collections::HashSet, sync::Mutex};

use tokio::sync::watch;

use super::recognizer::{ExecutionProfile, PreparedModel};
use crate::services::voice::download::VoiceCatalogEntry;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct ModelKey {
    pub(super) asr_id: String,
    asr_revision: String,
    pub(super) vad_id: String,
    vad_revision: String,
    profile: ExecutionProfile,
}

impl ModelKey {
    pub(super) fn new(
        asr: &VoiceCatalogEntry,
        vad: &VoiceCatalogEntry,
        profile: ExecutionProfile,
    ) -> Self {
        Self {
            asr_id: asr.id.clone(),
            asr_revision: asr.revision.clone(),
            vad_id: vad.id.clone(),
            vad_revision: vad.revision.clone(),
            profile,
        }
    }

    pub(super) fn uses(&self, model_id: &str) -> bool {
        self.asr_id == model_id || self.vad_id == model_id
    }

    #[cfg(test)]
    pub(super) fn fixture(asr_id: &str, vad_id: &str) -> Self {
        Self {
            asr_id: asr_id.into(),
            asr_revision: "a".repeat(40),
            vad_id: vad_id.into(),
            vad_revision: format!("sha256:{}", "b".repeat(64)),
            profile: ExecutionProfile::cpu(2).unwrap(),
        }
    }
}

pub(super) struct LoadedModel {
    pub(super) key: ModelKey,
    pub(super) model: PreparedModel,
}

#[derive(Default)]
pub(super) struct LifecycleState {
    pub(super) loaded: Option<LoadedModel>,
    pub(super) occupied: Option<ModelKey>,
    pub(super) verified: HashSet<(String, String)>,
    pub(super) generation: u64,
    pub(super) closing: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct UnloadPlan {
    pub(super) generation: u64,
    pub(super) deadline: Option<tokio::time::Instant>,
}

pub(super) struct LifecycleInner {
    pub(super) state: Mutex<LifecycleState>,
    pub(super) plan: watch::Sender<UnloadPlan>,
}

pub(super) fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
