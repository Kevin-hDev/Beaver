use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const SESSION_ALLOW_TTL: Duration = Duration::from_secs(3600);
const MAX_ALLOWED_SESSIONS: usize = 64;
const MAX_ALLOWED_TOOLS_PER_SESSION: usize = 16;
const NO_SESSION_ALLOW: &[&str] = &["bash", "search_mcp_tools"];

static ALLOWED: LazyLock<Mutex<HashMap<String, HashMap<String, Instant>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn is_allowed(session_id: &str, tool: &str) -> bool {
    if NO_SESSION_ALLOW.contains(&tool) {
        return false;
    }
    let mut guard = ALLOWED.lock().await;
    prune_expired(&mut guard);
    let session_map = match guard.get_mut(session_id) {
        Some(map) => map,
        None => return false,
    };
    match session_map.get(tool) {
        Some(granted_at) if granted_at.elapsed() < SESSION_ALLOW_TTL => true,
        Some(_) => {
            session_map.remove(tool);
            false
        }
        None => false,
    }
}

pub async fn mark_allowed(session_id: &str, tool: &str) {
    if NO_SESSION_ALLOW.contains(&tool) || !valid_key(session_id) || !valid_key(tool) {
        return;
    }
    let mut allowed = ALLOWED.lock().await;
    prune_expired(&mut allowed);
    if !allowed.contains_key(session_id) && allowed.len() >= MAX_ALLOWED_SESSIONS {
        return;
    }
    let tools = allowed.entry(session_id.to_string()).or_default();
    if !tools.contains_key(tool) && tools.len() >= MAX_ALLOWED_TOOLS_PER_SESSION {
        return;
    }
    tools.insert(tool.to_string(), Instant::now());
}

pub async fn clear_session(session_id: &str) {
    ALLOWED.lock().await.remove(session_id);
}

fn prune_expired(allowed: &mut HashMap<String, HashMap<String, Instant>>) {
    allowed.retain(|_, tools| {
        tools.retain(|_, granted_at| granted_at.elapsed() < SESSION_ALLOW_TTL);
        !tools.is_empty()
    });
}

fn valid_key(value: &str) -> bool {
    !value.is_empty() && !value.contains('\0') && value.chars().count() <= 128
}

#[cfg(test)]
pub async fn allowed_tool_count_for_test(session_id: &str) -> usize {
    ALLOWED
        .lock()
        .await
        .get(session_id)
        .map_or(0, HashMap::len)
}

#[cfg(test)]
#[path = "permission_allow_cache_tests.rs"]
mod tests;
