use super::types::{ProviderWindow, RemoteData};
use reqwest::header::HeaderMap;

pub fn parse_rate_headers(connection_id: &str, headers: &HeaderMap) -> Option<RemoteData> {
    let mut windows = Vec::new();
    match connection_id {
        "cerebras" => push_cerebras(&mut windows, headers),
        "groq" => push_groq(&mut windows, headers),
        _ => return None,
    }
    if windows.is_empty() {
        return None;
    }
    Some(RemoteData {
        windows,
        fetched_at: chrono::Utc::now().timestamp(),
        ..Default::default()
    })
}

fn push_cerebras(windows: &mut Vec<ProviderWindow>, headers: &HeaderMap) {
    push_window(
        windows,
        "requests_day",
        headers,
        "x-ratelimit-limit-requests-day",
        "x-ratelimit-remaining-requests-day",
        "x-ratelimit-reset-requests-day",
    );
    push_window(
        windows,
        "tokens_minute",
        headers,
        "x-ratelimit-limit-tokens-minute",
        "x-ratelimit-remaining-tokens-minute",
        "x-ratelimit-reset-tokens-minute",
    );
}

fn push_groq(windows: &mut Vec<ProviderWindow>, headers: &HeaderMap) {
    push_window(
        windows,
        "requests_limit",
        headers,
        "x-ratelimit-limit-requests",
        "x-ratelimit-remaining-requests",
        "x-ratelimit-reset-requests",
    );
    push_window(
        windows,
        "tokens_limit",
        headers,
        "x-ratelimit-limit-tokens",
        "x-ratelimit-remaining-tokens",
        "x-ratelimit-reset-tokens",
    );
}

fn push_window(
    windows: &mut Vec<ProviderWindow>,
    label: &str,
    headers: &HeaderMap,
    limit_key: &str,
    remaining_key: &str,
    reset_key: &str,
) {
    let Some(limit) = number(headers, limit_key) else {
        return;
    };
    let remaining = number(headers, remaining_key);
    let used = remaining.map(|value| (limit - value).max(0.0));
    let used_percent =
        used.and_then(|value| (limit > 0.0).then_some((value / limit * 100.0).clamp(0.0, 100.0)));
    let resets_at = text(headers, reset_key)
        .and_then(parse_duration_seconds)
        .map(|seconds| {
            chrono::Utc::now()
                .timestamp()
                .saturating_add(seconds as i64)
        });
    windows.push(ProviderWindow {
        label_code: label.to_string(),
        group_code: None,
        group_name: None,
        used,
        limit: Some(limit),
        remaining,
        used_percent,
        resets_at,
    });
}

fn number(headers: &HeaderMap, key: &str) -> Option<f64> {
    let value = text(headers, key)?.parse::<f64>().ok()?;
    (value.is_finite() && (0.0..=1e15).contains(&value)).then_some(value)
}

fn text<'a>(headers: &'a HeaderMap, key: &str) -> Option<&'a str> {
    headers
        .get(key)?
        .to_str()
        .ok()
        .filter(|value| value.len() <= 64)
}

fn parse_duration_seconds(value: &str) -> Option<u64> {
    if let Ok(seconds) = value.parse::<f64>() {
        return valid_seconds(seconds);
    }
    let mut total = 0.0;
    let mut number = String::new();
    for character in value.chars() {
        if character.is_ascii_digit() || character == '.' {
            number.push(character);
            continue;
        }
        if number.is_empty() {
            return None;
        }
        let amount = number.parse::<f64>().ok()?;
        number.clear();
        total += amount
            * match character {
                's' => 1.0,
                'm' => 60.0,
                'h' => 3_600.0,
                'd' => 86_400.0,
                _ => return None,
            };
    }
    if !number.is_empty() {
        return None;
    }
    valid_seconds(total)
}

fn valid_seconds(value: f64) -> Option<u64> {
    (value.is_finite() && (0.0..=31_536_000.0).contains(&value)).then_some(value.ceil() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compound_reset_duration() {
        assert_eq!(parse_duration_seconds("2m59.5s"), Some(180));
        assert_eq!(parse_duration_seconds("7.2s"), Some(8));
        assert_eq!(parse_duration_seconds("secret"), None);
    }
}
