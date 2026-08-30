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
    AnthropicBlocks,
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

    /// La bascule reste centralisée : le transport doit être validé en réel et
    /// le mode doit appartenir au contrat exact du modèle.
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

pub fn reasoning_mode_is_live(
    route_id: RouteId,
    model_id: &str,
    reasoning_mode: ReasoningModeId,
) -> bool {
    if reasoning_mode == ReasoningModeId::Off {
        return false;
    }
    let Some(route) = ACTIVE_ROUTES
        .iter()
        .find(|route| route.route_id == route_id)
    else {
        return false;
    };
    [
        ContinuationUse::UserContinuation,
        ContinuationUse::ToolContinuation,
    ]
    .into_iter()
    .all(|continuation_use| {
        resolve_model_policy(route, model_id, reasoning_mode, continuation_use)
            .is_some_and(|policy| policy.activation == ActivationState::LiveValidated)
    })
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
    let policy = resolve_model_policy(
        route,
        &target.model_id,
        target.reasoning_mode,
        target.continuation_use,
    )?;
    Some(ReplayPolicy {
        contract_id: Some(route.contract_id),
        adapter: Some(route.adapter),
        requirement: policy.requirement,
        activation: policy.activation,
    })
}

#[derive(Clone, Copy)]
struct ResolvedPolicy {
    requirement: ReplayRequirement,
    activation: ActivationState,
}

fn resolve_model_policy(
    route: &RouteContract,
    model_id: &str,
    reasoning_mode: ReasoningModeId,
    continuation_use: ContinuationUse,
) -> Option<ResolvedPolicy> {
    if let Some(policy) = route.models.iter().find(|policy| {
        policy.model_id == model_id
            && policy.reasoning_mode == reasoning_mode
            && policy.continuation_use == continuation_use
    }) {
        return Some(ResolvedPolicy {
            requirement: policy.requirement,
            activation: policy.activation,
        });
    }
    if reasoning_mode == ReasoningModeId::Off
        || !matches!(route.route_id, RouteId::Anthropic | RouteId::Qwen)
        || !model_advertises_mode(route.route_id, model_id, reasoning_mode)
    {
        return None;
    }
    // Model Studio peut activer plus de modèles qu'il ne sait en rejouer :
    // une capacité d'activation n'autorise jamais implicitement le rejeu.
    if route.route_id == RouteId::Qwen
        && !crate::services::llm::provider_model_lookup::supports_reasoning_replay(
            route.route_id.provider_id(),
            model_id,
        )
    {
        return Some(ResolvedPolicy {
            requirement: ReplayRequirement::Forbidden,
            activation: ActivationState::LiveValidated,
        });
    }
    // Anthropic et les modèles Model Studio explicitement compatibles valident
    // le transport une fois par route. Les modes restent propres au modèle.
    let transport = route.models.iter().find(|policy| {
        policy.continuation_use == continuation_use
            && policy.activation == ActivationState::LiveValidated
            && policy.requirement != ReplayRequirement::Forbidden
    })?;
    Some(ResolvedPolicy {
        requirement: transport.requirement,
        activation: transport.activation,
    })
}

fn model_advertises_mode(
    route_id: RouteId,
    model_id: &str,
    reasoning_mode: ReasoningModeId,
) -> bool {
    crate::services::llm::provider_model_lookup::resolve_local(route_id.provider_id(), model_id)
        .is_some_and(|capabilities| {
            capabilities.supports_thinking
                && capabilities
                    .reasoning_modes
                    .iter()
                    .any(|mode| ReasoningModeId::from_name(Some(mode)) == Some(reasoning_mode))
        })
}
