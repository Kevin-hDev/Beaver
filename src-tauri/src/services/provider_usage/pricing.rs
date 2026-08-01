use super::request_usage::RequestUsage;

#[derive(Debug, Clone, Copy, Default)]
pub struct ResolvedCost {
    pub micros: Option<u64>,
    pub exact: bool,
}

pub async fn resolve(connection_id: &str, model: &str, usage: &RequestUsage) -> ResolvedCost {
    if let Some(micros) = usage.exact_cost_usd_micros {
        return ResolvedCost {
            micros: Some(micros),
            exact: true,
        };
    }
    if connection_id.ends_with("-oauth") {
        return ResolvedCost::default();
    }
    let provider = crate::services::llm::route::canonical_provider_id(connection_id);
    if matches!(provider, "openai" | "openrouter")
        && crate::services::llm::providers::openai::is_gpt_56(model)
    {
        return ResolvedCost::default();
    }
    let Some(pricing) = crate::services::llm::model_pricing::lookup(provider, model).await else {
        return ResolvedCost::default();
    };
    let Some(input_tokens) = usage.input_tokens else {
        return ResolvedCost::default();
    };
    let Some(output_tokens) = usage.output_tokens else {
        return ResolvedCost::default();
    };
    let cached = usage.cached_input_tokens.unwrap_or(0).min(input_tokens);
    let written = usage.cache_write_input_tokens.unwrap_or(0);
    let Some(fresh) = input_tokens
        .checked_sub(cached)
        .and_then(|remaining| remaining.checked_sub(written))
    else {
        return ResolvedCost::default();
    };
    let Some(input_rate) = pricing.input_cost_per_token else {
        return ResolvedCost::default();
    };
    let Some(output_rate) = pricing.output_cost_per_token else {
        return ResolvedCost::default();
    };
    let input_cost = fresh as f64 * input_rate;
    let cache_rate = pricing.cache_read_input_token_cost.unwrap_or(input_rate);
    let write_cost = if written == 0 {
        0.0
    } else {
        let Some(write_rate) = pricing.cache_creation_input_token_cost else {
            return ResolvedCost::default();
        };
        written as f64 * write_rate
    };
    let output_cost = output_tokens as f64 * output_rate;
    let dollars = input_cost + cached as f64 * cache_rate + write_cost + output_cost;
    if !dollars.is_finite() || !(0.0..=1_000_000.0).contains(&dollars) {
        return ResolvedCost::default();
    }
    ResolvedCost {
        micros: Some((dollars * 1_000_000.0).round() as u64),
        exact: false,
    }
}
