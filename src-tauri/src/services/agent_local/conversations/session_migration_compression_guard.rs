use serde_json::Value;

pub(super) fn migrate_v3(value: &mut Value) -> Result<(), String> {
    let object = value
        .as_object_mut()
        .ok_or_else(super::session_limits::invalid_session)?;
    if object.get("schema_version").and_then(Value::as_u64) != Some(3) {
        return Err(super::session_limits::invalid_session());
    }
    object.insert(
        "schema_version".into(),
        Value::from(super::session_limits::CURRENT_SESSION_SCHEMA_VERSION),
    );
    object.insert(
        "automatic_compression_guard".into(),
        serde_json::json!({
            "last_attempt": null,
            "consecutive_failures": 0,
            "suspended": false
        }),
    );
    Ok(())
}

pub(super) fn validate(
    guard: &super::types_session::AutomaticCompressionGuard,
) -> Result<(), String> {
    if guard.consecutive_failures > 3 || (guard.suspended && guard.consecutive_failures < 3) {
        return Err(super::session_limits::invalid_session());
    }
    let Some(attempt) = guard.last_attempt.as_ref() else {
        return guard
            .is_empty()
            .then_some(())
            .ok_or_else(super::session_limits::invalid_session);
    };
    super::session_migration_ids::validate_id(&attempt.top_level_turn_id)?;
    super::session_migration_ids::validate_id(&attempt.last_message_id)?;
    if let Some(id) = attempt.last_checkpoint_message_id.as_deref() {
        super::session_migration_ids::validate_id(id)?;
    }
    if attempt.message_count as usize > super::session_limits::MAX_MESSAGES_PER_SESSION
        || attempt.provider_id.is_empty()
        || attempt.provider_id.len() > 64
        || attempt.model_id.is_empty()
        || attempt.model_id.len() > 256
        || attempt.profile_id.is_empty()
        || attempt.profile_id.len() > 64
        || attempt.profile_revision == 0
        || attempt.global_selection_revision == 0
    {
        return Err(super::session_limits::invalid_session());
    }
    Ok(())
}

pub(super) fn normalize_for_read(session: &mut super::types_session::AgentSession) {
    if validate(&session.automatic_compression_guard).is_err() {
        log::warn!("automatic_compression_guard_invalid_reset");
        session.automatic_compression_guard = Default::default();
    }
}
