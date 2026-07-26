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
    let provider = if connection_id == "codex-oauth" {
        "openai"
    } else {
        crate::services::llm::route::canonical_provider_id(connection_id)
    };
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
    let fresh = input_tokens.saturating_sub(cached);
    let Some(input_rate) = pricing.input_cost_per_token else {
        return ResolvedCost::default();
    };
    let Some(output_rate) = pricing.output_cost_per_token else {
        return ResolvedCost::default();
    };
    let input_cost = fresh as f64 * input_rate;
    let cache_rate = pricing.cache_read_input_token_cost.unwrap_or(input_rate);
    let output_cost = output_tokens as f64 * output_rate;
    let dollars = input_cost + cached as f64 * cache_rate + output_cost;
    if !dollars.is_finite() || !(0.0..=1_000_000.0).contains(&dollars) {
        return ResolvedCost::default();
    }
    ResolvedCost {
        micros: Some((dollars * 1_000_000.0).round() as u64),
        exact: false,
    }
}
