use serde_json::Value;

pub(super) fn remote_output_limit(model: &Value) -> Option<u32> {
    [
        model.pointer("/top_provider/max_completion_tokens"),
        model.pointer("/limits/max_completion_tokens"),
        model.get("max_output_tokens"),
        model.get("max_completion_tokens"),
        model.get("max_tokens"),
    ]
    .into_iter()
    .flatten()
    .filter_map(super::model_metadata::positive_u32)
    .min()
}

pub(super) fn remote_context(model: &Value, use_top_provider_limit: bool) -> Option<u32> {
    let advertised = [
        &model["context_length"],
        &model["context_window"],
        &model["max_context_length"],
    ]
    .into_iter()
    .find_map(super::model_metadata::positive_u32);
    if !use_top_provider_limit {
        return advertised;
    }
    if advertised.is_none() {
        return super::model_metadata::positive_u32(&model["top_provider"]["context_length"]);
    }
    let top_provider =
        super::model_metadata::positive_u32(&model["top_provider"]["context_length"]);
    match (advertised, top_provider) {
        (Some(advertised), Some(top_provider)) => Some(advertised.min(top_provider)),
        (Some(advertised), None) => Some(advertised),
        (None, Some(top_provider)) => Some(top_provider),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_uses_the_smallest_positive_remote_limit() {
        let model = serde_json::json!({
            "top_provider": {"max_completion_tokens": 8_000},
            "max_output_tokens": 4_000,
            "max_tokens": 0
        });

        assert_eq!(remote_output_limit(&model), Some(4_000));
    }

    #[test]
    fn gateway_context_is_capped_without_changing_direct_provider_context() {
        let model = serde_json::json!({
            "context_length": 128_000,
            "top_provider": {"context_length": 64_000}
        });

        assert_eq!(remote_context(&model, true), Some(64_000));
        assert_eq!(remote_context(&model, false), Some(128_000));
    }
}
