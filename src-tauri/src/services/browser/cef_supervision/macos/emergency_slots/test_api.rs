use super::*;

impl MacEmergencySlots {
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
