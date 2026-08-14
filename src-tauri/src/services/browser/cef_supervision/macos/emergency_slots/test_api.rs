use super::*;

impl MacEmergencySlots {
    pub(in super::super) fn begin_closing_for_test(
        &self,
        helper_exit_ticks: u64,
        ultimate_ticks: u64,
    ) -> Result<(), ()> {
        self.begin_closing(helper_exit_ticks, ultimate_ticks)
    }

    pub(in super::super) fn closing_deadlines_for_test(&self) -> Option<(u64, u64)> {
        self.closing
            .get()
            .map(|deadlines| (deadlines.helper_exit, deadlines.ultimate))
    }

    pub(in super::super) fn force_pass_with_for_test(
        &self,
        actions: &impl MacProcessActions,
        now_ticks: u64,
    ) -> Result<(), ()> {
        self.force_pass_with(actions, now_ticks)
    }

    pub(in super::super) fn force_final_pass_with_for_test(
        &self,
        actions: &impl MacProcessActions,
        now_ticks: u64,
    ) -> Result<(), ()> {
        self.force_final_pass_with(actions, now_ticks)
    }

    pub(in super::super) fn normal_observation_for_test(
        &self,
        slot: usize,
        generation: u64,
        observation: MacProcessObservation,
        now_ticks: u64,
    ) -> Result<Option<MacLivenessDecision>, CefUnavailableCategory> {
        self.observation_for_test(slot, generation, observation, now_ticks)
    }

    pub(in super::super) fn emergency_observation_for_test(
        &self,
        slot: usize,
        generation: u64,
        observation: MacProcessObservation,
        now_ticks: u64,
    ) -> Result<Option<MacLivenessDecision>, CefUnavailableCategory> {
        self.observation_for_test(slot, generation, observation, now_ticks)
    }

    fn observation_for_test(
        &self,
        slot: usize,
        generation: u64,
        observation: MacProcessObservation,
        now_ticks: u64,
    ) -> Result<Option<MacLivenessDecision>, CefUnavailableCategory> {
        let mut target = match self.write(slot) {
            Some(target) => target,
            None => return Ok(None),
        };
        if !target
            .as_ref()
            .is_some_and(|entry| entry.generation == generation)
        {
            return Ok(None);
        }
        self.apply_observation(&mut target, observation, now_ticks)
    }
}
