use chrono::{DateTime, NaiveDate, NaiveDateTime};

const MAX_TEXT_CHARS: usize = 500;
const MAX_DATE_CHARS: usize = 80;

pub(super) fn clean_text(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > MAX_TEXT_CHARS || trimmed.contains('\0') {
        return Err("Annotation invalide".into());
    }
    Ok(trimmed.to_string())
}

pub(super) fn clean_date(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().count() > MAX_DATE_CHARS
        || trimmed.contains('\0')
        || trimmed.contains(['\n', '\r'])
        || !supported_date(trimmed)
    {
        return Err("Date d'annotation invalide".into());
    }
    Ok(trimmed.to_string())
}

fn supported_date(value: &str) -> bool {
    DateTime::parse_from_rfc3339(value).is_ok()
        || NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").is_ok()
        || NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_are_strict_and_text_is_bounded() {
        assert_eq!(clean_date("2026-08-01").unwrap(), "2026-08-01");
        assert!(clean_date("tomorrow").is_err());
        assert!(clean_text(&"x".repeat(MAX_TEXT_CHARS + 1)).is_err());
    }
}
