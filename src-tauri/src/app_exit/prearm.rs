use super::{policy, state, AppExitCoordinator, ExitIntent};

impl AppExitCoordinator {
    pub(super) fn prearm_request(&self, intent: ExitIntent, exit_code: i32) -> bool {
        let _guard = match self.begin_lock.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        match self.state.phase() {
            state::ShutdownPhase::Running => self.prepare_exit_locked(intent, exit_code).is_some(),
            state::ShutdownPhase::Closing | state::ShutdownPhase::ReadyToExit => true,
        }
    }

    pub(super) fn prepare_exit_locked(
        &self,
        intent: ExitIntent,
        exit_code: i32,
    ) -> Option<(policy::ShutdownTimeline, ExitIntent, i32)> {
        if let Some(timeline) = self.timeline.get().copied() {
            let owned_intent = self.intent.get().copied()?;
            let owned_exit_code = self.exit_code.get().copied()?;
            return Some((timeline, owned_intent, owned_exit_code));
        }
        let timeline =
            policy::ShutdownTimeline::from_origin(std::time::Instant::now(), self.policy);
        if self.intent.set(intent).is_err()
            || self.exit_code.set(exit_code).is_err()
            || self.timeline.set(timeline).is_err()
            || !self.ultimate.arm(timeline.ultimate_deadline(), exit_code)
        {
            return None;
        }
        Some((timeline, intent, exit_code))
    }
}
