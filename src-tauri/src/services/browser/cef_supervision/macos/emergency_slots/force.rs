use super::*;
use crate::services::browser::cef_supervision::constants::CEF_SLOT_CAPACITY;
use crate::services::browser::cef_supervision::macos::process_state::{
    MacSignalObservation, MacSignalResult,
};

impl MacEmergencySlots {
    pub(in crate::services::browser::cef_supervision::macos) fn force_pass(
        &self,
    ) -> Result<(), ()> {
        let now_ticks = super::super::clock::now_ticks().map_err(|_| ())?;
        self.force_pass_with(&MacSystemProcessActions, now_ticks)
    }

    pub(in crate::services::browser::cef_supervision::macos) fn force_final_pass(
        &self,
    ) -> Result<(), ()> {
        let now_ticks = super::super::clock::now_ticks().map_err(|_| ())?;
        self.force_final_pass_with(&MacSystemProcessActions, now_ticks)
    }

    pub(super) fn force_pass_with(
        &self,
        actions: &impl MacProcessActions,
        now_ticks: u64,
    ) -> Result<(), ()> {
        self.run_force_pass_with(actions, now_ticks, true)
    }

    pub(super) fn force_final_pass_with(
        &self,
        actions: &impl MacProcessActions,
        now_ticks: u64,
    ) -> Result<(), ()> {
        self.run_force_pass_with(actions, now_ticks, false)
    }

    fn run_force_pass_with(
        &self,
        actions: &impl MacProcessActions,
        now_ticks: u64,
        allow_new_unknown: bool,
    ) -> Result<(), ()> {
        let mut failed = false;
        for slot in 0..CEF_SLOT_CAPACITY {
            if self
                .force_slot(slot, actions, now_ticks, allow_new_unknown)
                .is_err()
            {
                failed = true;
            }
        }
        (!failed).then_some(()).ok_or(())
    }

    fn force_slot(
        &self,
        slot: usize,
        actions: &impl MacProcessActions,
        now_ticks: u64,
        allow_new_unknown: bool,
    ) -> Result<(), ()> {
        let Some(mut target) = self.write(slot) else {
            return Ok(());
        };
        let Some(observation) = target
            .as_ref()
            .map(|entry| actions.observe(&entry.identity))
        else {
            return Ok(());
        };
        match self.apply_force_observation(
            &mut target,
            observation,
            now_ticks,
            allow_new_unknown,
        )? {
            None | Some(MacLivenessDecision::Stopped | MacLivenessDecision::Pending) => Ok(()),
            Some(MacLivenessDecision::Expired) => Err(()),
            Some(MacLivenessDecision::Alive) => {
                self.force_alive(&mut target, actions, now_ticks, allow_new_unknown)
            }
        }
    }

    fn force_alive(
        &self,
        target: &mut Option<MacEmergencyEntry>,
        actions: &impl MacProcessActions,
        now_ticks: u64,
        allow_new_unknown: bool,
    ) -> Result<(), ()> {
        let Some(revalidation) = target
            .as_ref()
            .map(|entry| actions.revalidate_before_signal(&entry.identity))
        else {
            return Ok(());
        };
        match revalidation {
            MacSignalObservation::Stopped => {
                self.apply_stopped(target, now_ticks)?;
                Ok(())
            }
            MacSignalObservation::Unknown => match self.apply_force_observation(
                target,
                MacProcessObservation::Unknown,
                now_ticks,
                allow_new_unknown,
            )? {
                Some(MacLivenessDecision::Pending | MacLivenessDecision::Stopped) | None => Ok(()),
                Some(MacLivenessDecision::Expired | MacLivenessDecision::Alive) => Err(()),
            },
            MacSignalObservation::Ready => match target
                .as_ref()
                .map(|entry| actions.signal_group(&entry.identity))
                .unwrap_or(Ok(MacSignalResult::Stopped))
            {
                Ok(MacSignalResult::Sent) => Ok(()),
                Ok(MacSignalResult::Stopped) => {
                    self.apply_stopped(target, now_ticks)?;
                    Ok(())
                }
                Err(_) => Err(()),
            },
        }
    }

    fn apply_force_observation(
        &self,
        target: &mut Option<MacEmergencyEntry>,
        observation: MacProcessObservation,
        now_ticks: u64,
        allow_new_unknown: bool,
    ) -> Result<Option<MacLivenessDecision>, ()> {
        if observation == MacProcessObservation::Unknown
            && !allow_new_unknown
            && target
                .as_ref()
                .is_some_and(|entry| !entry.liveness.has_unknown_budget())
        {
            return Ok(Some(MacLivenessDecision::Expired));
        }
        self.apply_observation(target, observation, now_ticks)
            .map_err(|_| ())
    }

    fn apply_stopped(
        &self,
        target: &mut Option<MacEmergencyEntry>,
        now_ticks: u64,
    ) -> Result<(), ()> {
        match self
            .apply_observation(target, MacProcessObservation::Stopped, now_ticks)
            .map_err(|_| ())?
        {
            Some(MacLivenessDecision::Stopped) | None => Ok(()),
            _ => Err(()),
        }
    }
}
