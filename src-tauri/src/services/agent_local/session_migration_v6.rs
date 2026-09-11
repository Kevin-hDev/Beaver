use serde_json::Value;

pub(super) const SCHEMA_VERSION: u16 = 6;

pub(super) fn migrate(value: &mut Value) -> Result<(), String> {
    let object = value
        .as_object_mut()
        .ok_or_else(super::session_limits::invalid_session)?;
    if object.get("schema_version").and_then(Value::as_u64)
        != Some(super::session_migration_v5::SCHEMA_VERSION.into())
    {
        return Err(super::session_limits::invalid_session());
    }
    object.insert("schema_version".into(), Value::from(SCHEMA_VERSION));
    object.remove("context_tokens");
    object.insert(
        "context_usage".into(),
        serde_json::to_value(super::context_usage_record::ContextUsageRecord::default())
            .map_err(|_| super::session_limits::invalid_session())?,
    );
    Ok(())
}
