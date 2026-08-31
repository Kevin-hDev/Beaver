use crate::services::agent_local::types_ollama::ChatMessage;

pub(super) fn estimate_native_continuation_tokens(
    provider_id: &str,
    messages: &[ChatMessage],
) -> usize {
    let Some(route) =
        crate::services::reasoning_continuity::contract::RouteId::from_provider_id(provider_id)
    else {
        return 0;
    };
    messages.iter().fold(0usize, |total, message| {
        let bytes = message.continuation.as_ref().and_then(|envelope| {
            (envelope.completion
                == crate::services::reasoning_continuity::envelope::CompletionState::Complete
                && envelope.source.route_id == route
                && crate::services::reasoning_continuity::registry::route_contract(route)
                    == Some(envelope.contract_id))
            .then(|| serde_json::to_vec(envelope).ok())
            .flatten()
        });
        total.saturating_add(bytes.map_or(0, |value| value.len().saturating_add(3) / 4))
    })
}
