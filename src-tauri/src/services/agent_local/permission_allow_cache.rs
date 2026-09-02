use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const SESSION_ALLOW_TTL: Duration = Duration::from_secs(3600);
const MAX_ALLOWED_SESSIONS: usize = 64;
const MAX_ALLOWED_TOOLS_PER_SESSION: usize = 16;
const NO_SESSION_ALLOW: &[&str] = &["bash", "bash_control", "search_mcp_tools"];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AllowedTool {
    extension_id: String,
    tool_name: String,
}

static ALLOWED: LazyLock<Mutex<HashMap<String, HashMap<AllowedTool, Instant>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn is_allowed(session_id: &str, tool: &str) -> bool {
    is_allowed_key(session_id, "", tool).await
}

pub(super) async fn is_extension_allowed(
    session_id: &str,
    extension_id: &str,
    tool_name: &str,
) -> bool {
    is_allowed_key(session_id, extension_id, tool_name).await
}

async fn is_allowed_key(session_id: &str, extension_id: &str, tool_name: &str) -> bool {
    let key = AllowedTool {
        extension_id: extension_id.to_string(),
        tool_name: tool_name.to_string(),
    };
    if extension_id.is_empty() && NO_SESSION_ALLOW.contains(&tool_name) {
        return false;
    }
    let mut guard = ALLOWED.lock().await;
    prune_expired(&mut guard);
    let session_map = match guard.get_mut(session_id) {
        Some(map) => map,
        None => return false,
    };
    match session_map.get(&key) {
        Some(granted_at) if granted_at.elapsed() < SESSION_ALLOW_TTL => true,
        Some(_) => {
            session_map.remove(&key);
            false
        }
        None => false,
    }
}

pub async fn mark_allowed(session_id: &str, tool: &str) {
    mark_allowed_key(session_id, "", tool).await;
}

pub(super) async fn mark_extension_allowed(session_id: &str, extension_id: &str, tool_name: &str) {
    mark_allowed_key(session_id, extension_id, tool_name).await;
}

async fn mark_allowed_key(session_id: &str, extension_id: &str, tool_name: &str) {
    if (extension_id.is_empty() && NO_SESSION_ALLOW.contains(&tool_name))
        || !valid_key(session_id)
        || (!extension_id.is_empty() && !valid_key(extension_id))
        || !valid_key(tool_name)
    {
        return;
    }
    let mut allowed = ALLOWED.lock().await;
    prune_expired(&mut allowed);
    if !allowed.contains_key(session_id) && allowed.len() >= MAX_ALLOWED_SESSIONS {
        return;
    }
    let tools = allowed.entry(session_id.to_string()).or_default();
    let key = AllowedTool {
        extension_id: extension_id.to_string(),
        tool_name: tool_name.to_string(),
    };
    if !tools.contains_key(&key) && tools.len() >= MAX_ALLOWED_TOOLS_PER_SESSION {
        return;
    }
    tools.insert(key, Instant::now());
}

pub async fn clear_session(session_id: &str) {
    ALLOWED.lock().await.remove(session_id);
}

pub(super) async fn clear_extension(extension_id: &str) {
    if !valid_key(extension_id) {
        return;
    }
    let mut allowed = ALLOWED.lock().await;
    allowed.retain(|_, tools| {
        tools.retain(|key, _| key.extension_id != extension_id);
        !tools.is_empty()
    });
}

pub(super) async fn clear_all_extensions() {
    let mut allowed = ALLOWED.lock().await;
    clear_all_extensions_in(&mut allowed);
}

fn clear_all_extensions_in(allowed: &mut HashMap<String, HashMap<AllowedTool, Instant>>) {
    allowed.retain(|_, tools| {
        tools.retain(|key, _| key.extension_id.is_empty());
        !tools.is_empty()
    });
}

fn prune_expired(allowed: &mut HashMap<String, HashMap<AllowedTool, Instant>>) {
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
    ALLOWED.lock().await.get(session_id).map_or(0, HashMap::len)
}

#[cfg(test)]
#[path = "permission_allow_cache_tests.rs"]
mod tests;
