use super::types::{ProviderBalance, ProviderWindow, RemoteData, BALANCE_LIMIT, WINDOW_LIMIT};

const GROUP_CODE_LIMIT: usize = 64;
const GROUP_NAME_LIMIT: usize = 96;

pub fn parse(body: &serde_json::Value) -> RemoteData {
    let mut windows = Vec::new();
    if let Some(rate_limit) = body.get("rate_limit").and_then(non_null) {
        push_windows(rate_limit, Some("general".into()), None, &mut windows);
    }
    if let Some(additional) = body["additional_rate_limits"].as_array() {
        for item in additional.iter().take(WINDOW_LIMIT) {
            let Some(rate_limit) = item.get("rate_limit").and_then(non_null) else {
                continue;
            };
            let group_code = item.get("metered_feature").and_then(group_code);
            let group_name = item.get("limit_name").and_then(group_name);
            push_windows(rate_limit, group_code, group_name, &mut windows);
        }
    }
    finish(windows, balances(body))
}

fn push_windows(
    rate_limit: &serde_json::Value,
    group_code: Option<String>,
    group_name: Option<String>,
    windows: &mut Vec<ProviderWindow>,
) {
    for key in ["primary_window", "secondary_window"] {
        let Some(window) = rate_limit.get(key).and_then(non_null) else {
            continue;
        };
        let percent = finite(window["used_percent"].as_f64());
        let seconds = window["limit_window_seconds"].as_u64().unwrap_or(0);
        windows.push(ProviderWindow {
            label_code: duration_label(seconds).into(),
            group_code: group_code.clone(),
            group_name: group_name.clone(),
            used: percent,
            limit: Some(100.0),
            remaining: percent.map(|value| (100.0 - value).max(0.0)),
            used_percent: percent,
            resets_at: timestamp(&window["reset_at"]),
        });
        if windows.len() >= WINDOW_LIMIT {
            break;
        }
    }
}

fn balances(body: &serde_json::Value) -> Vec<ProviderBalance> {
    let mut balances = Vec::new();
    if let Some(amount) = body
        .pointer("/credits/balance")
        .and_then(super::remote_api::decimal_value)
    {
        balances.push(ProviderBalance {
            label_code: "remaining_credits".into(),
            amount,
            currency: "USD".into(),
        });
    }
    if let Some(count) = body
        .pointer("/rate_limit_reset_credits/available_count")
        .and_then(serde_json::Value::as_u64)
    {
        balances.push(ProviderBalance {
            label_code: "reset_credits".into(),
            amount: count.min(1_000_000).to_string(),
            currency: "CREDITS".into(),
        });
    }
    balances
}

fn finish(mut windows: Vec<ProviderWindow>, mut balances: Vec<ProviderBalance>) -> RemoteData {
    windows.truncate(WINDOW_LIMIT);
    balances.truncate(BALANCE_LIMIT);
    RemoteData {
        windows,
        balances,
        fetched_at: chrono::Utc::now().timestamp(),
        ..Default::default()
    }
}

fn group_code(value: &serde_json::Value) -> Option<String> {
    let value = value.as_str()?;
    (value.len() <= GROUP_CODE_LIMIT
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_-".contains(character)))
    .then(|| value.to_string())
}

fn group_name(value: &serde_json::Value) -> Option<String> {
    let value = value.as_str()?;
    (!value.is_empty()
        && value.chars().count() <= GROUP_NAME_LIMIT
        && !value.chars().any(char::is_control))
    .then(|| value.to_string())
}

fn duration_label(seconds: u64) -> &'static str {
    if (14_400..=21_600).contains(&seconds) {
        "rolling_five_hours"
    } else if seconds >= 518_400 {
        "weekly"
    } else {
        "provider_limit"
    }
}

fn non_null(value: &serde_json::Value) -> Option<&serde_json::Value> {
    (!value.is_null()).then_some(value)
}

fn finite(value: Option<f64>) -> Option<f64> {
    value.filter(|value| value.is_finite() && *value >= 0.0 && *value <= 100.0)
}

fn timestamp(value: &serde_json::Value) -> Option<i64> {
    value.as_i64().and_then(super::types::valid_reset_timestamp)
}
