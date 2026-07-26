use super::types_session::SubagentLastActivity;
use chrono::Utc;
use serde_json::Value;

const MAX_LABEL_CHARS: usize = 80;
const MAX_DETAIL_CHARS: usize = 220;

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
    let lock = super::session_store::lock_session(session_id).await;
    let _guard = lock.lock().await;
    let Ok(mut session) = super::session_store::get(session_id).await else {
        return;
    };
    if session.parent_session_id.is_none() {
        return;
    }
    session.subagent_last_activity = Some(SubagentLastActivity {
        kind: bounded(kind, MAX_LABEL_CHARS),
        label: bounded(label, MAX_LABEL_CHARS),
        detail: detail.map(|value| bounded(value, MAX_DETAIL_CHARS)),
        updated_at: Utc::now(),
    });
    session.updated_at = Some(Utc::now());
    let _ = super::session_store::save(&session).await;
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
    use super::bounded;

    #[test]
    fn bounded_collapses_whitespace_and_limits_chars() {
        assert_eq!(bounded("  a   b  c  ", 4), "a b ");
    }
}
