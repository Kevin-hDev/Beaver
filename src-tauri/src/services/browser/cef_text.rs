use cef::CefString;

pub(super) fn bounded_cef_text(value: &CefString, max_units: usize) -> Option<String> {
    let utf16 = value.as_slice()?;
    let end = utf16.len().min(max_units);
    Some(String::from_utf16_lossy(&utf16[..end]))
}

pub(super) fn validated_cef_url(value: &CefString) -> Option<String> {
    let utf16 = value.as_slice()?;
    if utf16.len() > super::url_policy::MAX_BROWSER_URL_LENGTH {
        return None;
    }
    let raw = String::from_utf16_lossy(utf16);
    super::url_policy::validate_browser_url(&raw)
        .ok()
        .map(|url| url.as_str().to_owned())
}
