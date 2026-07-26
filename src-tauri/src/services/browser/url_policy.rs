pub(super) const MAX_BROWSER_URL_LENGTH: usize = 2_048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ValidatedUrl(String);

impl ValidatedUrl {
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

pub(super) fn validate_browser_url(input: &str) -> Result<ValidatedUrl, ()> {
    if input.is_empty()
        || input.len() > MAX_BROWSER_URL_LENGTH
        || input.trim() != input
        || input.contains('\\')
        || input.chars().any(char::is_control)
    {
        return Err(());
    }
    let (_, authority_and_path) = input.split_once("://").ok_or(())?;
    let authority_end = authority_and_path
        .find(['/', '?', '#'])
        .unwrap_or(authority_and_path.len());
    if authority_end == 0 {
        return Err(());
    }
    let parsed = url::Url::parse(input).map_err(|_| ())?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(());
    }
    let normalized = parsed.to_string();
    if normalized.len() > MAX_BROWSER_URL_LENGTH {
        return Err(());
    }
    Ok(ValidatedUrl(normalized))
}
