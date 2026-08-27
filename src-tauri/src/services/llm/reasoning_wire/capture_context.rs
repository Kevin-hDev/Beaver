use crate::services::reasoning_continuity::contract::{
    CredentialScope, ReasoningModeId, ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::envelope::ReasoningSource;

/// Provenance fixée avant lecture du premier événement provider.
#[derive(Debug, Clone)]
pub(crate) struct ReasoningCaptureContext {
    pub route_id: RouteId,
    pub model_id: String,
    pub credential_scope: CredentialScope,
    pub reasoning_mode: ReasoningModeId,
}

impl ReasoningCaptureContext {
    pub(crate) fn from_target(target: &ReplayTarget) -> Self {
        Self {
            route_id: target.route_id,
            model_id: target.model_id.clone(),
            credential_scope: target.credential_scope.clone(),
            reasoning_mode: target.reasoning_mode,
        }
    }

    pub(super) fn source(&self) -> ReasoningSource {
        ReasoningSource {
            route_id: self.route_id,
            model_id: self.model_id.clone(),
            credential_scope: self.credential_scope.clone(),
            reasoning_mode: self.reasoning_mode,
        }
    }
}
