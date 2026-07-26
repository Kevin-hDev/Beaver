use url::Url;

const MAX_URL_BYTES: usize = 2_048;
const MAX_ASSET_NAME_BYTES: usize = 128;

pub(crate) fn is_safe_version(value: &str) -> bool {
    parse_version(value).is_some()
}

pub(crate) fn strict_version_gt(remote: &str, local: &str) -> bool {
    matches!(
        (parse_version(remote), parse_version(local)),
        (Some(remote), Some(local)) if remote > local
    )
}

fn parse_version(value: &str) -> Option<[u64; 3]> {
    if value.is_empty() || value.len() > 32 {
        return None;
    }
    let mut parts = value.split('.');
    let mut parsed = [0_u64; 3];
    for target in &mut parsed {
        let part = parts.next()?;
        if part.is_empty()
            || (part.len() > 1 && part.starts_with('0'))
            || !part.bytes().all(|byte| byte.is_ascii_digit())
        {
            return None;
        }
        *target = part.parse().ok()?;
    }
    parts.next().is_none().then_some(parsed)
}

pub(super) fn strict_url(raw: &str, host: &str) -> Option<Url> {
    let authority = raw.strip_prefix("https://")?.split('/').next()?;
    let lower = raw.to_ascii_lowercase();
    if authority != host
        || raw.is_empty()
        || raw.len() > MAX_URL_BYTES
        || raw.contains("..")
        || raw.contains('\\')
        || lower.contains("%2e")
        || raw.chars().any(|character| character.is_control())
    {
        return None;
    }
    let url = Url::parse(raw).ok()?;
    (url.scheme() == "https"
        && url.host_str() == Some(host)
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && url.query().is_none()
        && url.fragment().is_none())
    .then_some(url)
}

pub(super) fn safe_repository_part(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

pub(crate) fn is_safe_asset_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ASSET_NAME_BYTES
        && !value.contains("..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

pub(super) fn redirect_target_is_allowed(target: &Url, host: &str) -> bool {
    let path = target.path().to_ascii_lowercase();
    target.as_str().len() <= MAX_URL_BYTES
        && target.scheme() == "https"
        && target.host_str() == Some(host)
        && target.username().is_empty()
        && target.password().is_none()
        && target.port().is_none()
        && target.fragment().is_none()
        && target.path() != "/"
        && !path.contains("..")
        && !path.contains("%2e")
}
