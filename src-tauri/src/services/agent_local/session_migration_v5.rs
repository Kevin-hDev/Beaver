use serde_json::Value;

pub(super) const SCHEMA_VERSION: u16 = 5;

pub(super) fn migrate(value: &mut Value) -> Result<(), String> {
    let object = value
        .as_object_mut()
        .ok_or_else(super::session_limits::invalid_session)?;
    object.insert("schema_version".into(), Value::from(SCHEMA_VERSION));
    Ok(())
}
