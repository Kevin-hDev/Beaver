use super::types_session::SubagentLastActivity;
use chrono::Utc;
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};

const MAX_LABEL_CHARS: usize = 80;
const MAX_DETAIL_CHARS: usize = 220;
const MAX_SESSION_CACHE: usize = 64;
const NEGATIVE_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(1);

struct CachedSession {
    id: String,
    is_subagent: bool,
    checked_at: std::time::Instant,
}

static SUBAGENT_CACHE: LazyLock<Mutex<VecDeque<CachedSession>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

pub async fn record_status(session_id: &str, label: &str, detail: Option<&str>) {
    record(session_id, "status", label, detail).await;
}

pub async fn record_tool_started(session_id: &str, tool: &str, summary: Option<&Value>) {
    record(
        session_id,
        "tool",
        &format!("{tool} démarré"),
        value_detail(summary).as_deref(),
    )
    .await;
}

pub async fn record_tool_completed(
    session_id: &str,
    tool: &str,
    summary: Option<&Value>,
    is_error: bool,
) {
    let label = if is_error {
        format!("{tool} terminé avec erreur")
    } else {
        format!("{tool} terminé")
    };
    record(session_id, "tool", &label, value_detail(summary).as_deref()).await;
}

async fn record(session_id: &str, kind: &str, label: &str, detail: Option<&str>) {
    if !is_subagent(session_id).await {
        return;
    }
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let Ok(mut session) = super::session_store::get(session_id).await else {
        return;
    };
    session.subagent_last_activity = Some(SubagentLastActivity {
        kind: bounded(kind, MAX_LABEL_CHARS),
        label: bounded(label, MAX_LABEL_CHARS),
        detail: detail.map(|value| bounded(value, MAX_DETAIL_CHARS)),
        updated_at: Utc::now(),
    });
    session.updated_at = Some(Utc::now());
    let _ = super::session_store::save(&session).await;
}

async fn is_subagent(session_id: &str) -> bool {
    if super::session_store::validate_session_id(session_id).is_err() {
        return false;
    }
    if let Some(result) = cached_subagent(session_id) {
        return result;
    }
    let result = super::session_store::get(session_id)
        .await
        .is_ok_and(|session| session.parent_session_id.is_some());
    remember_subagent(session_id, result);
    result
}

fn cached_subagent(session_id: &str) -> Option<bool> {
    let mut cache = SUBAGENT_CACHE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let position = cache.iter().position(|entry| entry.id == session_id)?;
    let entry = cache.remove(position)?;
    if !entry.is_subagent && entry.checked_at.elapsed() >= NEGATIVE_CACHE_TTL {
        return None;
    }
    let result = entry.is_subagent;
    cache.push_back(entry);
    Some(result)
}

fn remember_subagent(session_id: &str, is_subagent: bool) {
    let mut cache = SUBAGENT_CACHE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if let Some(position) = cache.iter().position(|entry| entry.id == session_id) {
        cache.remove(position);
    }
    if cache.len() >= MAX_SESSION_CACHE {
        cache.pop_front();
    }
    cache.push_back(CachedSession {
        id: session_id.to_string(),
        is_subagent,
        checked_at: std::time::Instant::now(),
    });
}

fn value_detail(value: Option<&Value>) -> Option<String> {
    value.map(|value| {
        value
            .as_str()
            .map(ToString::to_string)
            .unwrap_or_else(|| value.to_string())
    })
}

fn bounded(value: &str, max_chars: usize) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max_chars)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{bounded, cached_subagent, remember_subagent};

    #[test]
    fn bounded_collapses_whitespace_and_limits_chars() {
        assert_eq!(bounded("  a   b  c  ", 4), "a b ");
    }

    #[test]
    fn only_confirmed_subagents_are_cached() {
        let ordinary = uuid::Uuid::new_v4().to_string();
        let subagent = uuid::Uuid::new_v4().to_string();

        remember_subagent(&ordinary, false);
        remember_subagent(&subagent, true);

        assert_eq!(cached_subagent(&ordinary), Some(false));
        assert_eq!(cached_subagent(&subagent), Some(true));
    }
}
