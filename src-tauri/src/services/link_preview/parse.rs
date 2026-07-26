pub fn extract_domain(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_string()
}

pub fn extract_og(html: &str, property: &str) -> Option<String> {
    let pattern = format!(r#"property="{property}""#);
    let alt_pattern = format!(r#"property='{property}'"#);
    find_meta_content(html, &pattern).or_else(|| find_meta_content(html, &alt_pattern))
}

pub fn extract_meta_name(html: &str, name: &str) -> Option<String> {
    let pattern = format!(r#"name="{name}""#);
    find_meta_content(html, &pattern)
}

fn find_meta_content(html: &str, attr_pattern: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let pos = lower.find(&attr_pattern.to_lowercase())?;
    let tag_start = lower[..pos].rfind('<')?;
    let tag_end = lower[pos..].find('>')? + pos;
    if !html.is_char_boundary(tag_start) || !html.is_char_boundary(tag_end + 1) {
        return None;
    }
    let tag = &html[tag_start..=tag_end];
    extract_attr(tag, "content")
}

pub fn extract_tag(html: &str, tag_name: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let open = format!("<{tag_name}");
    let close = format!("</{tag_name}>");
    let start = lower.find(&open)?;
    let content_start = lower[start..].find('>')? + start + 1;
    let end = lower[content_start..].find(&close)? + content_start;
    if !html.is_char_boundary(content_start) || !html.is_char_boundary(end) {
        return None;
    }
    let text = html[content_start..end].trim();
    if text.is_empty() {
        None
    } else {
        Some(decode_entities(text))
    }
}

pub fn extract_favicon(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    for pattern in &["rel=\"icon\"", "rel=\"shortcut icon\"", "rel='icon'"] {
        if let Some(pos) = lower.find(pattern) {
            let tag_start = lower[..pos].rfind('<')?;
            let tag_end = pos + lower[pos..].find('>')?;
            if !html.is_char_boundary(tag_start) || !html.is_char_boundary(tag_end + 1) {
                continue;
            }
            let tag = &html[tag_start..=tag_end];
            if let Some(href) = extract_attr(tag, "href") {
                return Some(href);
            }
        }
    }
    None
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let needle = format!("{attr}=");
    let pos = lower.find(&needle)? + needle.len();
    if !tag.is_char_boundary(pos) {
        return None;
    }
    let rest = &tag[pos..];
    let (quote, start) = if rest.starts_with('"') {
        ('"', 1)
    } else if rest.starts_with('\'') {
        ('\'', 1)
    } else {
        return None;
    };
    let end = rest[start..].find(quote)? + start;
    let val = rest[start..end].trim();
    if val.is_empty() {
        None
    } else {
        Some(decode_entities(val))
    }
}

pub fn resolve_url(href: &str, base: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else if href.starts_with("//") {
        format!("https:{href}")
    } else if href.starts_with('/') {
        format!("{base}{href}")
    } else {
        format!("{base}/{href}")
    }
}

fn decode_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn og_title() {
        let html = r#"<html><head><meta property="og:title" content="Mon Site"></head></html>"#;
        assert_eq!(extract_og(html, "og:title"), Some("Mon Site".into()));
    }

    #[test]
    fn og_missing() {
        assert_eq!(extract_og("<html></html>", "og:title"), None);
    }

    #[test]
    fn title_tag() {
        let html = "<html><head><title>Hello World</title></head></html>";
        assert_eq!(extract_tag(html, "title"), Some("Hello World".into()));
    }

    #[test]
    fn favicon_link() {
        let html = r#"<link rel="icon" href="/img/favicon.png">"#;
        assert_eq!(extract_favicon(html), Some("/img/favicon.png".into()));
    }

    #[test]
    fn resolve_absolute() {
        assert_eq!(
            resolve_url("https://x.com/img.png", "https://x.com"),
            "https://x.com/img.png"
        );
    }

    #[test]
    fn resolve_relative() {
        assert_eq!(
            resolve_url("/img.png", "https://x.com"),
            "https://x.com/img.png"
        );
    }

    #[test]
    fn resolve_protocol_relative() {
        assert_eq!(
            resolve_url("//cdn.x.com/img.png", "https://x.com"),
            "https://cdn.x.com/img.png"
        );
    }

    #[test]
    fn domain_extraction() {
        assert_eq!(
            extract_domain("https://www.example.com/path"),
            "www.example.com"
        );
        assert_eq!(extract_domain("http://localhost:3000/"), "localhost");
    }

    #[test]
    fn entities() {
        assert_eq!(decode_entities("A &amp; B &lt;C&gt;"), "A & B <C>");
    }

    #[test]
    fn meta_name() {
        let html = r#"<meta name="description" content="A cool site">"#;
        assert_eq!(
            extract_meta_name(html, "description"),
            Some("A cool site".into())
        );
    }
}
