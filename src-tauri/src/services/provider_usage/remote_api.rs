use super::types::{ProviderBalance, ProviderWindow, RemoteData, BALANCE_LIMIT, WINDOW_LIMIT};

pub fn parse(connection_id: &str, body: &serde_json::Value) -> Option<RemoteData> {
    match connection_id {
        "openrouter" => Some(parse_openrouter(body)),
        "deepseek" => Some(parse_deepseek(body)),
        "moonshot" => Some(parse_moonshot(body)),
        _ => None,
    }
}

fn parse_openrouter(body: &serde_json::Value) -> RemoteData {
    let data = &body["data"];
    let mut windows = Vec::new();
    push_openrouter_window(&mut windows, "today", data["usage_daily"].as_f64());
    push_openrouter_window(&mut windows, "seven_days", data["usage_weekly"].as_f64());
    push_openrouter_window(&mut windows, "thirty_days", data["usage_monthly"].as_f64());
    let used = finite(data["usage"].as_f64());
    let limit = finite(data["limit"].as_f64());
    let remaining = signed(data["limit_remaining"].as_f64());
    if used.is_some() || limit.is_some() || remaining.is_some() {
        windows.push(ProviderWindow {
            label_code: "key_limit".into(),
            group_code: None,
            group_name: None,
            used,
            limit,
            remaining,
            used_percent: percent(used, limit),
            resets_at: None,
        });
    }
    let balances = remaining
        .map(|amount| ProviderBalance {
            label_code: "remaining_credits".into(),
            amount: decimal_number(amount),
            currency: "USD".into(),
        })
        .into_iter()
        .collect();
    finish(windows, balances, None)
}

fn push_openrouter_window(windows: &mut Vec<ProviderWindow>, label: &str, used: Option<f64>) {
    let Some(used) = finite(used) else { return };
    windows.push(ProviderWindow {
        label_code: label.into(),
        group_code: None,
        group_name: None,
        used: Some(used),
        ..Default::default()
    });
}

fn parse_deepseek(body: &serde_json::Value) -> RemoteData {
    let balances = body["balance_infos"]
        .as_array()
        .into_iter()
        .flatten()
        .take(BALANCE_LIMIT)
        .filter_map(|item| {
            let amount = decimal_value(&item["total_balance"])?;
            let currency = currency(&item["currency"])?;
            Some(ProviderBalance {
                label_code: "available_balance".into(),
                amount,
                currency,
            })
        })
        .collect();
    finish(Vec::new(), balances, None)
}

fn parse_moonshot(body: &serde_json::Value) -> RemoteData {
    let data = &body["data"];
    let mut balances = Vec::new();
    for (field, label) in [
        ("available_balance", "available_balance"),
        ("voucher_balance", "voucher_balance"),
        ("cash_balance", "cash_balance"),
    ] {
        if let Some(amount) = decimal_value(&data[field]) {
            balances.push(ProviderBalance {
                label_code: label.into(),
                amount,
                currency: "CNY".into(),
            });
        }
    }
    finish(Vec::new(), balances, None)
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

fn signed(value: Option<f64>) -> Option<f64> {
    value.filter(|value| value.is_finite() && value.abs() <= 1e15)
}

fn percent(used: Option<f64>, limit: Option<f64>) -> Option<f64> {
    match (used, limit) {
        (Some(used), Some(limit)) if limit > 0.0 => Some((used / limit * 100.0).clamp(0.0, 100.0)),
        _ => None,
    }
}

pub(super) fn decimal_value(value: &serde_json::Value) -> Option<String> {
    if let Some(raw) = value.as_str() {
        return valid_decimal(raw).then(|| raw.to_string());
    }
    signed(value.as_f64()).map(decimal_number)
}

fn decimal_number(value: f64) -> String {
    let formatted = format!("{value:.6}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn valid_decimal(value: &str) -> bool {
    if value.is_empty() || value.len() > 32 || value == "-" {
        return false;
    }
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    !unsigned.is_empty()
        && unsigned
            .parse::<f64>()
            .is_ok_and(|number| number.is_finite())
        && unsigned.chars().any(|character| character.is_ascii_digit())
}

fn currency(value: &serde_json::Value) -> Option<String> {
    let currency = value.as_str()?;
    (currency.len() == 3
        && currency
            .chars()
            .all(|character| character.is_ascii_uppercase()))
    .then(|| currency.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deepseek_balances_are_bounded_and_validated() {
        let body = json!({"balance_infos": [
            {"currency":"USD","total_balance":"12.50"},
            {"currency":"bad","total_balance":"secret"}
        ]});
        let parsed = parse("deepseek", &body).unwrap();
        assert_eq!(parsed.balances.len(), 1);
        assert_eq!(parsed.balances[0].amount, "12.50");
    }
}
