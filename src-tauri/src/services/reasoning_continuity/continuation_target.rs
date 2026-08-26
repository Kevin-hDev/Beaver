use super::contract::{ReasoningModeId, ReplayTarget, RouteId};
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
    Forbidden(NonReplayTarget),
}

impl ContinuationTarget {
    pub fn replay(&self) -> Option<&ReplayTarget> {
        match self {
            Self::Replay(target) => Some(target),
            Self::Forbidden(_) => None,
        }
    }

    pub fn route_id(&self) -> RouteId {
        match self {
            Self::Replay(target) => target.route_id,
            Self::Forbidden(target) => target.route_id,
        }
    }

    pub fn model_id(&self) -> &str {
        match self {
            Self::Replay(target) => &target.model_id,
            Self::Forbidden(target) => &target.model_id,
        }
    }

    pub fn reasoning_mode(&self) -> ReasoningModeId {
        match self {
            Self::Replay(target) => target.reasoning_mode,
            Self::Forbidden(target) => target.reasoning_mode,
        }
    }

    pub fn validate(&self) -> Result<(), LimitError> {
        match self {
            Self::Replay(target) => target.validate(),
            Self::Forbidden(target) => target.validate(),
        }
    }
}
