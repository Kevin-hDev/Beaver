use serde_json::Value;

pub(super) fn normalize_for_read(value: &mut Value) {
    let session_id = warning_session_id(value);
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
        log::warn!("session_context_usage_invalid_reset session_id={session_id}");
        object.remove("context_usage");
    }
}

pub(super) fn warning_session_id(value: &Value) -> String {
    value
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| uuid::Uuid::parse_str(id).is_ok())
        .unwrap_or("unknown")
        .to_string()
}
