pub(super) fn valid_provider_id(provider: &str) -> bool {
    !provider.is_empty()
        && provider.len() <= 32
        && provider
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || matches!(byte, b'-' | b'_'))
}

pub(super) fn valid_date(date: &str) -> bool {
    date.len() == 10 && chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok()
}

pub(super) fn valid_source_url(url: &str) -> bool {
    url.len() <= 512 && url.starts_with("https://") && !url.chars().any(char::is_control)
}
