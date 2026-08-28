use super::contract::{ContinuationUse, ContractId, ReasoningModeId, ReplayTarget, RouteId};
use super::registry_inventory::ACTIVE_ROUTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayRequirement {
    Required,
    Optional,
    Forbidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationState {
    Disabled,
    LiveValidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterId {
    OllamaNative,
    GeminiParts,
    MistralChunks,
    CerebrasReasoning,
    OpenRouterDetails,
    ResponsesLocal,
    ChatReasoning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelPolicy {
    pub model_id: &'static str,
    pub reasoning_mode: ReasoningModeId,
    pub continuation_use: ContinuationUse,
    pub requirement: ReplayRequirement,
    pub activation: ActivationState,
    pub fixture_id: Option<&'static str>,
    pub fixture_date: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteContract {
    pub route_id: RouteId,
    pub contract_id: ContractId,
    pub adapter: AdapterId,
    pub models: &'static [ModelPolicy],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayPolicy {
    contract_id: Option<ContractId>,
    adapter: Option<AdapterId>,
    requirement: ReplayRequirement,
    activation: ActivationState,
}

impl ReplayPolicy {
    pub const fn requirement(self) -> ReplayRequirement {
        self.requirement
    }

    pub const fn activation(self) -> ActivationState {
        self.activation
    }

    /// La bascule reste centralisée dans le registre : aucun adaptateur ne peut
    /// sérialiser une enveloppe tant que son couple exact n'est pas validé réel.
    pub(crate) fn live_adapter(self) -> Option<(ContractId, AdapterId)> {
        (self.activation == ActivationState::LiveValidated)
            .then_some((self.contract_id?, self.adapter?))
    }

    pub(crate) fn fixture_adapter(self) -> Option<(ContractId, AdapterId)> {
        Some((self.contract_id?, self.adapter?))
    }

    #[cfg(test)]
    pub(crate) const fn for_test(
        contract_id: Option<ContractId>,
        adapter: Option<AdapterId>,
        requirement: ReplayRequirement,
        activation: ActivationState,
    ) -> Self {
        Self {
            contract_id,
            adapter,
            requirement,
            activation,
        }
    }
}

pub fn active_routes() -> &'static [RouteContract] {
    ACTIVE_ROUTES
}

pub fn route_contract(route_id: RouteId) -> Option<ContractId> {
    ACTIVE_ROUTES
        .iter()
        .find(|entry| entry.route_id == route_id)
        .map(|entry| entry.contract_id)
}

pub fn replay_policy(target: &ReplayTarget) -> Option<ReplayPolicy> {
    if target.validate().is_err() {
        return None;
    }
    find_policy(ACTIVE_ROUTES, target)
}

#[cfg(test)]
pub(super) fn replay_policy_from_routes(
    routes: &[RouteContract],
    target: &ReplayTarget,
) -> Option<ReplayPolicy> {
    target.validate().ok()?;
    find_policy(routes, target)
}

fn find_policy(routes: &[RouteContract], target: &ReplayTarget) -> Option<ReplayPolicy> {
    let route = routes
        .iter()
        .find(|entry| entry.route_id == target.route_id)?;
    let policy = route.models.iter().find(|policy| {
        policy.model_id == target.model_id
            && policy.reasoning_mode == target.reasoning_mode
            && policy.continuation_use == target.continuation_use
    })?;
    Some(ReplayPolicy {
        contract_id: Some(route.contract_id),
        adapter: Some(route.adapter),
        requirement: policy.requirement,
        activation: policy.activation,
    })
}
