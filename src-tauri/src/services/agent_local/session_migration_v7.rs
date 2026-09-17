use serde_json::Value;

pub(super) const SCHEMA_VERSION: u16 = 7;

pub(super) fn migrate(value: &mut Value) -> Result<(), String> {
    let object = value
        .as_object_mut()
        .ok_or_else(super::session_limits::invalid_session)?;
    if object.get("schema_version").and_then(Value::as_u64)
        != Some(super::session_migration_v6::SCHEMA_VERSION.into())
    {
        return Err(super::session_limits::invalid_session());
    }
    object.insert("schema_version".into(), Value::from(SCHEMA_VERSION));
    object.insert("subagent_extension_owner".into(), Value::Null);
    Ok(())
}
