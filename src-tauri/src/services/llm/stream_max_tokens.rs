use super::model_registry_lookup;

pub async fn resolve(
    provider_id: &str,
    model_id: &str,
    requested: Option<u32>,
    auto_max_tokens: bool,
    provider_fallback: Option<u32>,
) -> Option<u32> {
    let runtime_limit = super::runtime_models::lookup(provider_id, model_id)
        .and_then(|model| model.max_output_tokens);
    let model_limit = match runtime_limit {
        Some(limit) => Some(limit),
        None => model_registry_lookup::max_output_tokens(provider_id, model_id).await,
    };
    choose(requested, model_limit, auto_max_tokens, provider_fallback)
}

fn choose(
    requested: Option<u32>,
    model_limit: Option<u32>,
    auto_max_tokens: bool,
    provider_fallback: Option<u32>,
) -> Option<u32> {
    match (requested, model_limit, auto_max_tokens) {
        (Some(value), Some(limit), _) => Some(value.min(limit)),
        (Some(value), None, _) => Some(value),
        (None, _, false) => None,
        (None, Some(limit), true) => Some(limit),
        (None, None, true) => provider_fallback,
    }
}

#[cfg(test)]
#[path = "stream_max_tokens_tests.rs"]
mod tests;
