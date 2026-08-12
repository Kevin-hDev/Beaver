use crate::models::{ClgoConfig, WakeupSchedule};
use crate::services::config as cfg;
use std::future::Future;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnceClaimOutcome {
    Claimed,
    Inactive,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum WakeupStepOutcome<T> {
    Completed(T),
    SkippedInactive,
    Cancelled,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum MissedOnceAction {
    LogMissed,
    Silent,
    LogClaimError(String),
}

pub(crate) async fn run_wakeup_steps<T, Claim, ClaimFuture, Dispatch, DispatchFuture>(
    is_once: bool,
    cancel: &CancellationToken,
    claim: Claim,
    dispatch: Dispatch,
) -> Result<WakeupStepOutcome<T>, String>
where
    Claim: FnOnce() -> ClaimFuture,
    ClaimFuture: Future<Output = Result<OnceClaimOutcome, String>>,
    Dispatch: FnOnce() -> DispatchFuture,
    DispatchFuture: Future<Output = Result<T, String>>,
{
    if cancel.is_cancelled() {
        return Err("cancelled".into());
    }
    let claimed_once = if is_once {
        match claim().await? {
            OnceClaimOutcome::Claimed => true,
            OnceClaimOutcome::Inactive => return Ok(WakeupStepOutcome::SkippedInactive),
        }
    } else {
        false
    };
    if cancel.is_cancelled() {
        return if claimed_once {
            Ok(WakeupStepOutcome::Cancelled)
        } else {
            Err("cancelled".into())
        };
    }
    match dispatch().await {
        Ok(value) => Ok(WakeupStepOutcome::Completed(value)),
        Err(_) if claimed_once && cancel.is_cancelled() => Ok(WakeupStepOutcome::Cancelled),
        Err(error) => Err(error),
    }
}

pub(crate) fn claim_once(id: &str) -> Result<OnceClaimOutcome, String> {
    let outcome = cfg::update_config(|config| Ok(claim_once_in(config, id)))?;
    if outcome == OnceClaimOutcome::Claimed {
        // La mutation doit réveiller la boucle afin que son instant suivant soit recalculé.
        super::notify_config_changed();
    }
    Ok(outcome)
}

pub(crate) fn claim_once_in(config: &mut ClgoConfig, id: &str) -> OnceClaimOutcome {
    let Some(wakeup) = config
        .scheduled_wakeups
        .iter_mut()
        .find(|wakeup| wakeup.id == id)
    else {
        return OnceClaimOutcome::Inactive;
    };
    if !wakeup.active || !matches!(wakeup.schedule, WakeupSchedule::Once { .. }) {
        return OnceClaimOutcome::Inactive;
    }
    wakeup.active = false;
    OnceClaimOutcome::Claimed
}

pub(crate) fn missed_once_action(outcome: Result<OnceClaimOutcome, String>) -> MissedOnceAction {
    match outcome {
        Ok(OnceClaimOutcome::Claimed) => MissedOnceAction::LogMissed,
        Ok(OnceClaimOutcome::Inactive) => MissedOnceAction::Silent,
        Err(error) => MissedOnceAction::LogClaimError(error),
    }
}
