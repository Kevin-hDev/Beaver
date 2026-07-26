use chrono::{DateTime, Datelike, Duration, Months, NaiveDate, NaiveDateTime, Weekday};

pub fn build_future_dates(last_date: &str, frequency: &str, horizon: u32) -> Vec<String> {
    let Some(last_datetime) = parse_input_datetime(last_date) else {
        return (1..=horizon).map(|index| format!("T+{index}")).collect();
    };
    let normalized = frequency.trim().to_uppercase();

    (1..=horizon)
        .map(|step| {
            shift_datetime(last_datetime, &normalized, step)
                .map(|value| format_output(value, last_date))
                .unwrap_or_else(|| format!("T+{step}"))
        })
        .collect()
}

pub fn parse_input_datetime(value: &str) -> Option<NaiveDateTime> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Some(parsed.naive_utc());
    }
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d %H:%M",
    ];
    for format in formats {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(value, format) {
            return Some(parsed);
        }
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(value, "%Y/%m/%d"))
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
}

fn shift_datetime(base: NaiveDateTime, frequency: &str, step: u32) -> Option<NaiveDateTime> {
    match frequency {
        "S" => Some(base + Duration::seconds(i64::from(step))),
        "T" | "MIN" => Some(base + Duration::minutes(i64::from(step))),
        "H" => Some(base + Duration::hours(i64::from(step))),
        "D" => Some(base + Duration::days(i64::from(step))),
        "B" => advance_business_days(base, step),
        "W" => Some(base + Duration::weeks(i64::from(step))),
        "M" => base.checked_add_months(Months::new(step)),
        "Q" => base.checked_add_months(Months::new(step.saturating_mul(3))),
        "Y" | "A" => base.checked_add_months(Months::new(step.saturating_mul(12))),
        _ => parse_compound_frequency(base, frequency, step),
    }
}

pub fn next_datetime(base: NaiveDateTime, frequency: &str) -> Option<NaiveDateTime> {
    shift_datetime(base, &frequency.trim().to_uppercase(), 1)
}

pub fn count_frequency_steps(
    start: NaiveDateTime,
    end: NaiveDateTime,
    frequency: &str,
    max_steps: usize,
) -> Option<usize> {
    if end <= start {
        return None;
    }
    let mut current = start;
    for step in 1..=max_steps {
        current = next_datetime(current, frequency)?;
        if current == end {
            return Some(step);
        }
        if current > end {
            return None;
        }
    }
    None
}

fn advance_business_days(base: NaiveDateTime, step: u32) -> Option<NaiveDateTime> {
    let mut current = base;
    let mut remaining = step;
    while remaining > 0 {
        current = current.checked_add_signed(Duration::days(1))?;
        if !matches!(current.weekday(), Weekday::Sat | Weekday::Sun) {
            remaining -= 1;
        }
    }
    Some(current)
}

fn parse_compound_frequency(
    base: NaiveDateTime,
    frequency: &str,
    step: u32,
) -> Option<NaiveDateTime> {
    let digits_len = frequency
        .chars()
        .take_while(|char| char.is_ascii_digit())
        .count();
    if digits_len == 0 || digits_len >= frequency.len() {
        return None;
    }
    let factor = frequency[..digits_len].parse::<u32>().ok()?;
    let unit = &frequency[digits_len..];
    let total = factor.saturating_mul(step);
    match unit {
        "S" => Some(base + Duration::seconds(i64::from(total))),
        "MIN" | "T" => Some(base + Duration::minutes(i64::from(total))),
        "H" => Some(base + Duration::hours(i64::from(total))),
        "D" => Some(base + Duration::days(i64::from(total))),
        "W" => Some(base + Duration::weeks(i64::from(total))),
        _ => None,
    }
}

fn format_output(value: NaiveDateTime, source: &str) -> String {
    if source.contains('T') {
        return value.format("%Y-%m-%dT%H:%M:%S").to_string();
    }
    if source.contains(':') {
        return value.format("%Y-%m-%d %H:%M:%S").to_string();
    }
    value.date().format("%Y-%m-%d").to_string()
}

#[cfg(test)]
#[path = "input_dates_tests.rs"]
mod tests;
