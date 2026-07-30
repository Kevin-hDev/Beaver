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
