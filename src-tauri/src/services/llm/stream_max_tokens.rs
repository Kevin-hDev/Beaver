use super::provider_model_lookup;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveError {
    ContextExhausted,
    InvalidLimit,
}

pub async fn resolve(
    provider_id: &str,
    model_id: &str,
    requested: Option<u32>,
    auto_max_tokens: bool,
    provider_fallback: Option<u32>,
    estimated_input_tokens: usize,
) -> Result<Option<u32>, ResolveError> {
    let local = provider_model_lookup::local_limits(provider_id, model_id);
    let runtime = super::runtime_models::lookup(provider_id, model_id);
    let registered = if local.is_none() {
        provider_model_lookup::limits(provider_id, model_id).await
    } else {
        None
    };
    let runtime_limits = runtime.map(|model| (model.context_length, model.max_output_tokens));
    let (context_window, model_limit) = select_sources(local, runtime_limits, registered);
    choose(
        requested,
        model_limit,
        auto_max_tokens,
        provider_fallback,
        context_window,
        estimated_input_tokens,
    )
}

fn select_sources(
    local: Option<provider_model_lookup::ModelLimits>,
    runtime: Option<(Option<u32>, Option<u32>)>,
    registered: Option<provider_model_lookup::ModelLimits>,
) -> (Option<u32>, Option<u32>) {
    if let Some(local) = local {
        return (local.context_window, local.max_output_tokens);
    }
    let (runtime_context, runtime_output) = runtime.unwrap_or_default();
    (
        runtime_context.or_else(|| registered.and_then(|limits| limits.context_window)),
        runtime_output.or_else(|| registered.and_then(|limits| limits.max_output_tokens)),
    )
}

fn choose(
    requested: Option<u32>,
    model_limit: Option<u32>,
    auto_max_tokens: bool,
    provider_fallback: Option<u32>,
    context_window: Option<u32>,
    estimated_input_tokens: usize,
) -> Result<Option<u32>, ResolveError> {
    if requested == Some(0) {
        return Err(ResolveError::InvalidLimit);
    }
    let candidate = match (requested, model_limit, auto_max_tokens) {
        (Some(value), Some(limit), _) => Some(value.min(limit)),
        (Some(value), None, _) => Some(value),
        (None, _, false) => None,
        (None, Some(limit), true) => Some(limit),
        (None, None, true) => provider_fallback.filter(|limit| *limit > 0),
    };
    let Some(context_window) = context_window.filter(|context| *context > 0) else {
        return Ok(candidate);
    };
    let input =
        u32::try_from(estimated_input_tokens).map_err(|_| ResolveError::ContextExhausted)?;
    let available = context_window
        .checked_sub(input)
        .filter(|available| *available > 0)
        .ok_or(ResolveError::ContextExhausted)?;
    Ok(candidate.map(|limit| limit.min(available)))
}

#[cfg(test)]
#[path = "stream_max_tokens_tests.rs"]
mod tests;
