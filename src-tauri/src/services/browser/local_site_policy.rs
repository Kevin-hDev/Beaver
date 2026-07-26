use super::local_site_types::MAX_LOCAL_TITLE_CHARS;
use url::{Host, Url};

const APP_DEV_PORT: u16 = 5173;
const OLLAMA_DEFAULT_PORT: u16 = 11_434;
const OLLAMA_RANGE: std::ops::RangeInclusive<u16> = 11_500..=11_599;
const FORECAST_RANGE: std::ops::RangeInclusive<u16> = 12_000..=12_099;

pub(super) fn is_internal_port(port: u16) -> bool {
    port == APP_DEV_PORT
        || port == OLLAMA_DEFAULT_PORT
        || OLLAMA_RANGE.contains(&port)
        || FORECAST_RANGE.contains(&port)
}

pub(super) fn is_allowed_local_url(url: &Url) -> bool {
    if url.as_str().len() > super::url_policy::MAX_BROWSER_URL_LENGTH
        || !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return false;
    }
    match url.host() {
        Some(Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(address)) => address == std::net::Ipv4Addr::LOCALHOST,
        Some(Host::Ipv6(address)) => address == std::net::Ipv6Addr::LOCALHOST,
        None => false,
    }
}

pub(super) fn looks_like_html(content_type: Option<&str>, body: &str) -> bool {
    let declared = content_type.is_some_and(|value| {
        let lower = value.to_ascii_lowercase();
        lower.starts_with("text/html") || lower.starts_with("application/xhtml+xml")
    });
    let prefix = body.trim_start().to_ascii_lowercase();
    declared || prefix.starts_with("<!doctype html") || prefix.starts_with("<html")
}

pub(super) fn is_internal_page(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    lower.contains("<title>searxng")
        || lower.contains("name=\"generator\" content=\"searxng")
        || ((lower.contains("cl-go") || lower.contains("beaver")) && lower.contains("tauri"))
        || lower.contains("cl-go forecast runtime")
        || lower.contains("beaver forecast runtime")
}

pub(super) fn clean_local_title(raw: Option<String>, port: u16) -> String {
    let mut cleaned = String::new();
    let mut length = 0;
    let mut inside_tag = false;
    let mut pending_space = false;
    for character in raw.as_deref().unwrap_or_default().chars() {
        if length == MAX_LOCAL_TITLE_CHARS {
            break;
        }
        if character == '<' {
            inside_tag = true;
            continue;
        }
        if character == '>' {
            inside_tag = false;
            continue;
        }
        if inside_tag {
            continue;
        }
        if character.is_whitespace() || character.is_control() {
            pending_space = !cleaned.is_empty();
            continue;
        }
        if pending_space && length < MAX_LOCAL_TITLE_CHARS {
            cleaned.push(' ');
            length += 1;
            pending_space = false;
        }
        if length < MAX_LOCAL_TITLE_CHARS {
            cleaned.push(character);
            length += 1;
        }
    }
    if cleaned.is_empty() {
        format!("localhost:{port}")
    } else {
        cleaned
    }
}
