use std::collections::HashSet;
use serde::Deserialize;
use serde_json::Value;

use super::session_limits::{self, CURRENT_SESSION_SCHEMA_VERSION};
use super::types_session::AgentSession;

#[derive(Deserialize)]
struct VersionProbe {
    schema_version: Option<u16>,
}

#[derive(Deserialize)]
struct V1SessionWire {
    messages: Vec<V1MessageWire>,
}

#[derive(Deserialize)]
struct V1MessageWire {
    role: String,
    #[serde(default)]
    tool_calls: Option<Vec<V1ToolCallWire>>,
}

#[derive(Deserialize)]
struct V1ToolCallWire {
    function: V1ToolFunctionWire,
}

#[derive(Deserialize)]
struct V1ToolFunctionWire {
    name: String,
}

pub(super) enum WireVersion {
    V1,
    V2,
    Future(u16),
}

pub(super) fn version(bytes: &[u8]) -> Result<WireVersion, String> {
    let probe: VersionProbe = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    Ok(match probe.schema_version {
        None | Some(1) => WireVersion::V1,
        Some(CURRENT_SESSION_SCHEMA_VERSION) => WireVersion::V2,
        Some(value) if value > CURRENT_SESSION_SCHEMA_VERSION => WireVersion::Future(value),
        Some(_) => return Err(invalid()),
    })
}

pub(super) fn parse_v1(bytes: &[u8]) -> Result<AgentSession, String> {
    let wire: V1SessionWire = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    validate_v1_shape(&wire)?;
    let mut value: Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    super::session_migration_ids::migrate_value(&mut value)?;
    let mut session = parse_v2_value(value)?;
    super::session_migration_legacy_history::repair(&mut session);
    super::conversation_history_validation::validate(&session.messages).map_err(|_| invalid())?;
    Ok(session)
}

pub(super) fn parse_v2(bytes: &[u8]) -> Result<AgentSession, String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    super::session_migration_ids::validate_required_v2_fields(&value)?;
    let mut session: AgentSession = serde_json::from_value(value).map_err(|_| invalid())?;
    degrade_unreplayable(&mut session);
    validate_v2_readable(&session)?;
    Ok(session)
}

pub(super) fn parse_future(bytes: &[u8], version: u16) -> Result<AgentSession, String> {
    let mut value: Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    super::session_migration_ids::normalize_future_view(&mut value)?;
    let mut session: AgentSession = serde_json::from_value(value).map_err(|_| invalid())?;
    session.schema_version = version;
    for message in &mut session.messages {
        message.continuation = None;
        message.replay_source = None;
    }
    Ok(session)
}

pub(super) fn validate_v2_writable(session: &AgentSession) -> Result<(), String> {
    validate_v2_readable(session)?;
    session_limits::validate_continuity(session)
}

fn validate_v2_readable(session: &AgentSession) -> Result<(), String> {
    if session.schema_version != CURRENT_SESSION_SCHEMA_VERSION
        || session.messages.len() > super::session_limits::MAX_MESSAGES_PER_SESSION
    {
        return Err(invalid());
    }
    for message in &session.messages {
        super::session_migration_ids::validate_id(&message.turn_id)?;
        if message.replay_source.as_ref().is_some_and(|source| {
            message.role != "user" || source.validate().is_err()
        }) {
            return Err(invalid());
        }
        super::conversation_skills::validate_persisted_references(
            message.skill_ids.as_deref(),
            message.skill_names.as_deref(),
        )
        .map_err(|_| invalid())?;
        if message.role == "tool" {
            super::session_migration_ids::validate_id(
                message.tool_call_id.as_deref().ok_or_else(invalid)?,
            )?;
        }
        if let Some(calls) = &message.tool_calls {
            if calls.len() > crate::services::reasoning_continuity::limits::MAX_TOOL_CALLS {
                return Err(invalid());
            }
            let mut ids = HashSet::with_capacity(calls.len());
            for call in calls {
                super::session_migration_ids::validate_id(&call.id)?;
                crate::services::reasoning_continuity::limits::validate_tool_name(
                    &call.function.name,
                )
                .map_err(|_| invalid())?;
                if !ids.insert(call.id.as_str()) {
                    return Err(invalid());
                }
            }
        }
    }
    Ok(())
}

fn degrade_unreplayable(session: &mut AgentSession) {
    for message in &mut session.messages {
        if message.role != "assistant"
            || message
                .continuation
                .as_ref()
                .is_some_and(|envelope| envelope.validate().is_err())
        {
            message.continuation = None;
        }
        if message.role != "user"
            || message
                .replay_source
                .as_ref()
                .is_some_and(|source| source.validate().is_err())
        {
            message.replay_source = None;
        }
        if super::conversation_skills::validate_persisted_references(
            message.skill_ids.as_deref(),
            message.skill_names.as_deref(),
        )
        .is_err()
        {
            message.skill_ids = None;
            message.skill_names = None;
        }
    }
    if session_limits::validate_continuity(session).is_err() {
        for message in &mut session.messages {
            message.continuation = None;
            message.replay_source = None;
        }
    }
}

fn validate_v1_shape(wire: &V1SessionWire) -> Result<(), String> {
    if wire.messages.len() > super::session_limits::MAX_MESSAGES_PER_SESSION {
        return Err(invalid());
    }
    for message in &wire.messages {
        if message.role.is_empty() || message.role.len() > 16 {
            return Err(invalid());
        }
        if let Some(calls) = &message.tool_calls {
            if calls.len() > crate::services::reasoning_continuity::limits::MAX_TOOL_CALLS
                || calls.iter().any(|call| call.function.name.is_empty())
            {
                return Err(invalid());
            }
        }
    }
    Ok(())
}

fn parse_v2_value(value: Value) -> Result<AgentSession, String> {
    let session: AgentSession = serde_json::from_value(value).map_err(|_| invalid())?;
    validate_v2_writable(&session)?;
    Ok(session)
}

fn invalid() -> String {
    session_limits::invalid_session()
}
