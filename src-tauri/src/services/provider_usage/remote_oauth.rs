use super::types::{ProviderBalance, ProviderWindow, RemoteData, BALANCE_LIMIT, WINDOW_LIMIT};

pub fn parse(connection_id: &str, body: &serde_json::Value) -> Option<RemoteData> {
    match connection_id {
        "codex-oauth" => Some(super::remote_codex::parse(body)),
        "moonshot-oauth" => parse_kimi(body),
        _ => None,
    }
}

fn parse_kimi(body: &serde_json::Value) -> Option<RemoteData> {
    let mut windows = Vec::new();
    if let Some(usage) = body.get("usage").and_then(serde_json::Value::as_object) {
        if let Some(window) = kimi_window(&serde_json::Value::Object(usage.clone()), "weekly") {
            windows.push(window);
        }
    }
    if let Some(limits) = body["limits"].as_array() {
        for item in limits.iter().take(WINDOW_LIMIT) {
            let detail = item
                .get("detail")
                .filter(|value| value.is_object())
                .unwrap_or(item);
            let duration = item.pointer("/window/duration").and_then(unsigned_integer);
            let unit = item
                .pointer("/window/timeUnit")
                .and_then(serde_json::Value::as_str);
            let label = if is_five_hour_window(duration, unit) {
                "rolling_five_hours"
            } else {
                "provider_limit"
            };
            if let Some(window) = kimi_window(detail, label) {
                windows.push(window);
            }
        }
    }
    let balances = super::remote_kimi_wallet::parse(body);
    if windows.is_empty() && balances.is_empty() {
        return None;
    }
    Some(finish(windows, balances, None))
}

fn kimi_window(value: &serde_json::Value, label: &str) -> Option<ProviderWindow> {
    let limit = number(&value["limit"]);
    let remaining = number(&value["remaining"]);
    let used = number(&value["used"]).or_else(|| match (limit, remaining) {
        (Some(limit), Some(remaining)) => Some((limit - remaining).max(0.0)),
        _ => None,
    });
    if used.is_none() && limit.is_none() {
        return None;
    }
    Some(ProviderWindow {
        label_code: label.into(),
        group_code: None,
        group_name: None,
        used,
        limit,
        remaining,
        used_percent: match (used, limit) {
            (Some(used), Some(limit)) if limit > 0.0 => {
                Some((used / limit * 100.0).clamp(0.0, 100.0))
            }
            _ => None,
        },
        resets_at: ["reset_at", "resetAt", "reset_time", "resetTime"]
            .iter()
            .find_map(|key| value.get(*key).and_then(parse_reset))
            .or_else(|| reset_after(value)),
    })
}

fn is_five_hour_window(duration: Option<u64>, unit: Option<&str>) -> bool {
    let Some(unit) = unit.filter(|value| value.len() <= 16) else {
        return false;
    };
    (duration == Some(300) && unit.eq_ignore_ascii_case("MINUTE"))
        || (duration == Some(5) && unit.eq_ignore_ascii_case("HOUR"))
}

fn finish(
    mut windows: Vec<ProviderWindow>,
    mut balances: Vec<ProviderBalance>,
    notice_code: Option<String>,
) -> RemoteData {
    windows.truncate(WINDOW_LIMIT);
    balances.truncate(BALANCE_LIMIT);
    RemoteData {
        windows,
        balances,
        notice_code,
        fetched_at: chrono::Utc::now().timestamp(),
        stale: false,
    }
}

fn finite(value: Option<f64>) -> Option<f64> {
    value.filter(|value| value.is_finite() && *value >= 0.0 && *value <= 1e15)
}

fn number(value: &serde_json::Value) -> Option<f64> {
    let parsed = value.as_f64().or_else(|| {
        let raw = value.as_str()?.trim();
        if raw.is_empty()
            || raw.len() > 32
            || !raw
                .chars()
                .all(|character| character.is_ascii_digit() || ".+-eE".contains(character))
        {
            return None;
        }
        raw.parse::<f64>().ok()
    });
    finite(parsed)
}

fn unsigned_integer(value: &serde_json::Value) -> Option<u64> {
    value.as_u64().or_else(|| {
        let raw = value.as_str()?;
        if raw.is_empty() || raw.len() > 20 || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        raw.parse::<u64>().ok()
    })
}

fn timestamp(value: &serde_json::Value) -> Option<i64> {
    value.as_i64().and_then(super::types::valid_reset_timestamp)
}

fn parse_reset(value: &serde_json::Value) -> Option<i64> {
    timestamp(value).or_else(|| {
        let raw = value.as_str().filter(|value| value.len() <= 64)?;
        if !raw.is_empty() && raw.bytes().all(|byte| byte.is_ascii_digit()) {
            return raw
                .parse::<i64>()
                .ok()
                .and_then(super::types::valid_reset_timestamp);
        }
        chrono::DateTime::parse_from_rfc3339(raw)
            .ok()
            .and_then(|date| super::types::valid_reset_timestamp(date.timestamp()))
    })
}

fn reset_after(value: &serde_json::Value) -> Option<i64> {
    let seconds = ["reset_in", "resetIn", "ttl", "window"]
        .iter()
        .find_map(|key| value.get(*key).and_then(parse_seconds))?;
    super::types::valid_reset_timestamp(
        chrono::Utc::now()
            .timestamp()
            .saturating_add(seconds as i64),
    )
}

fn parse_seconds(value: &serde_json::Value) -> Option<u64> {
    let seconds = value
        .as_u64()
        .or_else(|| value.as_str()?.parse::<u64>().ok())?;
    (seconds <= 31_536_000).then_some(seconds)
}
