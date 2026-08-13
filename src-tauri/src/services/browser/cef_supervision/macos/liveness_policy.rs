use super::super::constants::CEF_LIVENESS_UNKNOWN_TIMEOUT;
use super::process_state::MacProcessObservation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct MacLivenessState {
    unknown_deadline: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MacLivenessDecision {
    Alive,
    Stopped,
    Pending,
    Expired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MacLivenessError {
    Clock,
}

impl MacLivenessState {
    pub(super) const fn new() -> Self {
        Self {
            unknown_deadline: None,
        }
    }

    pub(super) fn apply(
        &mut self,
        observation: MacProcessObservation,
        now_ticks: u64,
        closing_cap_ticks: Option<u64>,
    ) -> Result<MacLivenessDecision, MacLivenessError> {
        if now_ticks == 0 || closing_cap_ticks == Some(0) {
            return Err(MacLivenessError::Clock);
        }

        match observation {
            MacProcessObservation::Alive => {
                self.unknown_deadline = None;
                Ok(MacLivenessDecision::Alive)
            }
            MacProcessObservation::Stopped => {
                self.unknown_deadline = None;
                Ok(MacLivenessDecision::Stopped)
            }
            MacProcessObservation::Unknown => {
                let timeout_ticks = u64::try_from(CEF_LIVENESS_UNKNOWN_TIMEOUT.as_nanos())
                    .map_err(|_| MacLivenessError::Clock)?;
                let local_deadline = now_ticks
                    .checked_add(timeout_ticks)
                    .ok_or(MacLivenessError::Clock)?;
                let candidate = closing_cap_ticks.map_or(local_deadline, |closing_cap| {
                    local_deadline.min(closing_cap)
                });
                let deadline = self
                    .unknown_deadline
                    .map_or(candidate, |existing| existing.min(candidate));
                self.unknown_deadline = Some(deadline);
                if now_ticks >= deadline {
                    Ok(MacLivenessDecision::Expired)
                } else {
                    Ok(MacLivenessDecision::Pending)
                }
            }
        }
    }
}
