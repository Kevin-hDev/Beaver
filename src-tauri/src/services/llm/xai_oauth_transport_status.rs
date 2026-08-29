pub(super) fn classify_status(
    policy: crate::services::llm::route_profile::ErrorPolicy,
    status: u16,
    body: &str,
    has_retry_after: bool,
) -> &'static str {
    if policy != crate::services::llm::route_profile::ErrorPolicy::XaiOauth {
        return "provider_request_rejected";
    }
    match status {
        401 => "oauth_reauthentication_required",
        403 => "provider_access_unavailable",
        429 if !has_retry_after
            && crate::services::llm::provider_error::safe_details(body)
                .error_code
                .as_deref()
                == Some("resource-exhausted") =>
        {
            "provider_quota_exhausted"
        }
        429 => "rate_limit",
        500..=599 => "provider_temporarily_unavailable",
        _ => "provider_request_rejected",
    }
}

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
