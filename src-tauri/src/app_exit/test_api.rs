use super::{emergency, policy, registry, state, ultimate, AppExitCoordinator, ExitIntent};
use std::sync::{Arc, Mutex, OnceLock};

impl AppExitCoordinator {
    pub(super) fn mark_ready(&self) -> bool {
        self.state.mark_ready()
    }

    pub(super) fn from_parts_for_test(
        policy: policy::ShutdownPolicy,
        ultimate: ultimate::UltimateExit,
    ) -> Self {
        Self {
            begin_lock: Mutex::new(()),
            state: Arc::new(state::ShutdownState::new()),
            registry: registry::AdmissionRegistry::new(),
            emergency: emergency::EmergencyInventory::new(),
            policy,
            timeline: OnceLock::new(),
            intent: OnceLock::new(),
            ultimate,
        }
    }

    pub(super) fn admit_for_test(
        &self,
    ) -> Result<registry::TrackedAdmission, registry::AdmissionError> {
        self.registry.try_admit()
    }

    pub(super) fn ultimate_is_armed_for_test(&self) -> bool {
        self.ultimate.is_armed_for_test()
    }

    pub(super) fn phase_for_test(&self) -> state::ShutdownPhase {
        self.state.phase()
    }

    pub(super) fn close_registry_for_test(&self) {
        assert!(self.registry.close());
    }

    pub(super) fn intent_for_test(&self) -> Option<ExitIntent> {
        self.intent.get().copied()
    }
}
