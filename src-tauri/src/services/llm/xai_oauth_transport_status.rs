pub(super) fn requires_responses_backend(request: &super::stream_http::RequestConfig<'_>) -> bool {
    use crate::services::reasoning_continuity::registry::{ActivationState, ReplayRequirement};

    super::reasoning_wire::replay::target_for_request(request.messages, request.continuation_target)
        .and_then(|target| target.replay().cloned())
        .and_then(|target| crate::services::reasoning_continuity::registry::replay_policy(&target))
        .is_some_and(|policy| {
            policy.activation() == ActivationState::LiveValidated
                && policy.requirement() == ReplayRequirement::Required
        })
}

pub(super) fn catalog_reasoning_mode<'a>(
    model: &'a crate::services::llm_oauth::XaiCatalogModel,
    requested_mode: Option<&'a str>,
) -> Option<&'a str> {
    let contract = model.reasoning_contract.as_ref()?;
    requested_mode
        .filter(|mode| contract.supports_mode(mode))
        .or_else(|| contract.default_mode_name())
}

pub(super) const fn backend_path(backend: crate::services::llm_oauth::XaiBackend) -> &'static str {
    match backend {
        crate::services::llm_oauth::XaiBackend::ChatCompletions => "/chat/completions",
        crate::services::llm_oauth::XaiBackend::Responses => "/responses",
    }
}
