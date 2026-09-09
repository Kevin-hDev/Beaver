use serde_json::Value;

pub(super) fn positive_u32(value: &Value) -> Option<u32> {
    u32::try_from(value.as_u64()?)
        .ok()
        .filter(|number| *number > 0)
}

pub(super) fn output_limit(model: &Value) -> Option<u32> {
    [
        model.pointer("/top_provider/max_completion_tokens"),
        model.pointer("/limits/max_completion_tokens"),
        model.get("max_output_tokens"),
        model.get("max_completion_tokens"),
        model.get("max_tokens"),
    ]
    .into_iter()
    .flatten()
    .find_map(positive_u32)
}

pub(super) fn has_zero_pricing(pricing: &Value) -> bool {
    let Some(prices) = pricing.as_object() else {
        return false;
    };
    let Some(prompt) = prices.get("prompt") else {
        return false;
    };
    let Some(completion) = prices.get("completion") else {
        return false;
    };
    price_is_zero(prompt) && price_is_zero(completion) && prices.values().all(price_is_zero)
}

fn price_is_zero(value: &Value) -> bool {
    let price = value
        .as_str()
        .and_then(|raw| raw.parse::<f64>().ok())
        .or_else(|| value.as_f64());
    price.is_some_and(|amount| amount.is_finite() && amount == 0.0)
}
