use super::model_registry;

pub async fn resolve(
    provider_id: &str,
    model_id: &str,
    requested: Option<u32>,
    provider_fallback: Option<u32>,
) -> Option<u32> {
    let model_limit = model_registry::lookup(provider_id, model_id)
        .await
        .and_then(|config| config.max_output_tokens);
    choose(requested, model_limit, provider_fallback)
}

fn choose(
    requested: Option<u32>,
    model_limit: Option<u32>,
    provider_fallback: Option<u32>,
) -> Option<u32> {
    match (requested, model_limit) {
        (Some(value), Some(limit)) => Some(value.min(limit)),
        (Some(value), None) => Some(value),
        (None, Some(limit)) => Some(limit),
        (None, None) => provider_fallback,
    }
}

#[cfg(test)]
#[path = "stream_max_tokens_tests.rs"]
mod tests;
