use super::{AutomationActor, AutomationError, AutomationOrigin};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub(super) const MAX_ACTORS: usize = 64;
const MAX_ID_CHARS: usize = 128;
static ACTORS: OnceLock<Mutex<HashMap<String, AutomationActor>>> = OnceLock::new();

#[derive(Debug)]
pub struct AutomationActorGuard {
    request_id: String,
}

pub fn register_actor(
    request_id: &str,
    actor: AutomationActor,
) -> Result<AutomationActorGuard, AutomationError> {
    validate_id(request_id)?;
    validate_id(&actor.session_or_channel_id)?;
    let mut actors = actors().lock().unwrap_or_else(|error| error.into_inner());
    if actors.len() >= MAX_ACTORS || actors.contains_key(request_id) {
        return Err(AutomationError::CapacityReached);
    }
    actors.insert(request_id.to_string(), actor);
    Ok(AutomationActorGuard {
        request_id: request_id.to_string(),
    })
}

pub fn actor_for(
    session_id: &str,
    is_gateway: bool,
    gateway_channel_key: Option<&str>,
    request_id: Option<&str>,
) -> Result<AutomationActor, AutomationError> {
    if let Some(actor) = request_id.and_then(registered_actor) {
        return Ok(actor);
    }
    validate_id(session_id)?;
    if !is_gateway {
        return Ok(AutomationActor {
            origin: AutomationOrigin::Session,
            session_or_channel_id: session_id.to_string(),
            current_automation_id: None,
        });
    }
    let channel = gateway_channel_key.ok_or(AutomationError::InvalidInput)?;
    if channel.is_empty() || channel.len() > 1_024 {
        return Err(AutomationError::InvalidInput);
    }
    Ok(AutomationActor {
        origin: AutomationOrigin::ExternalChannel,
        session_or_channel_id: channel_fingerprint(channel),
        current_automation_id: None,
    })
}

pub fn is_automation_request(request_id: &str) -> bool {
    registered_actor(request_id).is_some()
}

impl Drop for AutomationActorGuard {
    fn drop(&mut self) {
        actors()
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&self.request_id);
    }
}

fn registered_actor(request_id: &str) -> Option<AutomationActor> {
    actors()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(request_id)
        .cloned()
}

fn actors() -> &'static Mutex<HashMap<String, AutomationActor>> {
    ACTORS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn validate_id(value: &str) -> Result<(), AutomationError> {
    if value.is_empty()
        || value.chars().count() > MAX_ID_CHARS
        || value.chars().any(char::is_control)
    {
        return Err(AutomationError::InvalidInput);
    }
    Ok(())
}

fn channel_fingerprint(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    format!("gateway:{}", hex::encode(&digest[..16]))
}
