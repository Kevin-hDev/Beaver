use serde_json::Value;

pub(super) fn normalize_for_read(value: &mut Value) {
    let Some(object) = value.as_object_mut() else {
        return;
    };
    let invalid = object.get("context_usage").is_some_and(|raw| {
        match serde_json::from_value::<super::context_usage_record::ContextUsageRecord>(raw.clone())
        {
            Ok(record) => record.validate().is_err(),
            Err(_) => true,
        }
    });
    if invalid {
        log::warn!("session_context_usage_invalid_reset");
        object.remove("context_usage");
    }
}
