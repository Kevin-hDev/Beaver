use super::contract::{ContinuationUse, ReasoningModeId, ReplayTarget, RouteId};
use super::limits::{validate_model_id, LimitError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonReplayTarget {
    pub route_id: RouteId,
    pub model_id: String,
    pub reasoning_mode: ReasoningModeId,
}

impl NonReplayTarget {
    pub fn validate(&self) -> Result<(), LimitError> {
        validate_model_id(&self.model_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContinuationTarget {
    Replay(ReplayTarget),
    /// N'existe que dans le binaire debug et ne peut être obtenu que par la
    /// commande de fixture. Il autorise la preuve locale avant la bascule du
    /// registre, sans élargir le chat de production.
    #[cfg(debug_assertions)]
    FixtureCandidate(ReplayTarget),
    Forbidden(NonReplayTarget),
}

impl ContinuationTarget {
    pub fn replay(&self) -> Option<&ReplayTarget> {
        match self {
            Self::Replay(target) => Some(target),
            #[cfg(debug_assertions)]
            Self::FixtureCandidate(target) => Some(target),
            Self::Forbidden(_) => None,
        }
    }

    /// La provenance d'admission reste inchangée ; seul le type du tour est
    /// déterminé juste avant l'appel provider, après les éventuels outils.
    pub fn for_continuation_use(&self, continuation_use: ContinuationUse) -> Self {
        let replay = |target: &ReplayTarget| {
            let mut target = target.clone();
            target.continuation_use = continuation_use;
            target
        };
        match self {
            Self::Replay(target) => Self::Replay(replay(target)),
            #[cfg(debug_assertions)]
            Self::FixtureCandidate(target) => Self::FixtureCandidate(replay(target)),
            Self::Forbidden(target) => Self::Forbidden(target.clone()),
        }
    }

    pub const fn is_fixture_candidate(&self) -> bool {
        #[cfg(debug_assertions)]
        {
            matches!(self, Self::FixtureCandidate(_))
        }
        #[cfg(not(debug_assertions))]
        {
            false
        }
    }

    pub fn route_id(&self) -> RouteId {
        match self {
            Self::Replay(target) => target.route_id,
            #[cfg(debug_assertions)]
            Self::FixtureCandidate(target) => target.route_id,
            Self::Forbidden(target) => target.route_id,
        }
    }

    pub fn model_id(&self) -> &str {
        match self {
            Self::Replay(target) => &target.model_id,
            #[cfg(debug_assertions)]
            Self::FixtureCandidate(target) => &target.model_id,
            Self::Forbidden(target) => &target.model_id,
        }
    }

    pub fn reasoning_mode(&self) -> ReasoningModeId {
        match self {
            Self::Replay(target) => target.reasoning_mode,
            #[cfg(debug_assertions)]
            Self::FixtureCandidate(target) => target.reasoning_mode,
            Self::Forbidden(target) => target.reasoning_mode,
        }
    }

    pub fn validate(&self) -> Result<(), LimitError> {
        match self {
            Self::Replay(target) => target.validate(),
            #[cfg(debug_assertions)]
            Self::FixtureCandidate(target) => target.validate(),
            Self::Forbidden(target) => target.validate(),
        }
    }
}
