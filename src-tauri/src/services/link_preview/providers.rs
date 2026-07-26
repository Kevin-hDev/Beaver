use reqwest::Client;
use serde::Deserialize;

use super::{LinkPreview, TIMEOUT};

const MAX_OEMBED: usize = 64_000;

pub fn is_youtube(domain: &str) -> bool {
    domain == "www.youtube.com"
        || domain == "youtube.com"
        || domain == "youtu.be"
        || domain == "m.youtube.com"
}

pub async fn youtube_preview(url: &str, domain: &str) -> Result<LinkPreview, String> {
    let video_id = extract_youtube_id(url).ok_or("Preview unavailable")?;
    if !is_valid_video_id(&video_id) {
        return Err("Preview unavailable".into());
    }
    let oembed_url = format!(
        "https://www.youtube.com/oembed?url=https://www.youtube.com/watch?v={video_id}&format=json"
    );

    let client = Client::builder()
        .build()
        .map_err(|_| "Preview unavailable".to_string())?;
    let resp = client
        .get(&oembed_url)
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|_| "Preview unavailable".to_string())?;

    if !resp.status().is_success() {
        return Err("Preview unavailable".into());
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|_| "Preview unavailable".to_string())?;
    if bytes.len() > MAX_OEMBED {
        return Err("Preview unavailable".into());
    }
    let data: YouTubeOembed =
        serde_json::from_slice(&bytes).map_err(|_| "Preview unavailable".to_string())?;

    let image = format!("https://img.youtube.com/vi/{video_id}/hqdefault.jpg");

    Ok(LinkPreview {
        url: url.to_string(),
        domain: domain.to_string(),
        site_name: Some("YouTube".to_string()),
        title: Some(data.title),
        description: Some(data.author_name),
        image: Some(image),
        favicon: Some("https://www.youtube.com/favicon.ico".to_string()),
    })
}

fn is_valid_video_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 16
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn extract_youtube_id(url: &str) -> Option<String> {
    if url.contains("youtu.be/") {
        let after = url.split("youtu.be/").nth(1)?;
        let id = after.split(['?', '&', '/']).next()?;
        return if id.is_empty() {
            None
        } else {
            Some(id.to_string())
        };
    }
    if url.contains("v=") {
        let after = url.split("v=").nth(1)?;
        let id = after.split(['&', '#']).next()?;
        return if id.is_empty() {
            None
        } else {
            Some(id.to_string())
        };
    }
    if url.contains("/embed/") || url.contains("/v/") {
        let sep = if url.contains("/embed/") {
            "/embed/"
        } else {
            "/v/"
        };
        let after = url.split(sep).nth(1)?;
        let id = after.split(['?', '&', '/']).next()?;
        return if id.is_empty() {
            None
        } else {
            Some(id.to_string())
        };
    }
    None
}

#[derive(Deserialize)]
struct YouTubeOembed {
    title: String,
    author_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn youtube_standard() {
        assert_eq!(
            extract_youtube_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".into())
        );
    }

    #[test]
    fn youtube_short() {
        assert_eq!(
            extract_youtube_id("https://youtu.be/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".into())
        );
    }

    #[test]
    fn youtube_embed() {
        assert_eq!(
            extract_youtube_id("https://www.youtube.com/embed/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".into())
        );
    }

    #[test]
    fn youtube_with_params() {
        assert_eq!(
            extract_youtube_id("https://www.youtube.com/watch?v=abc123&t=42"),
            Some("abc123".into())
        );
    }

    #[test]
    fn youtube_mobile() {
        assert_eq!(
            extract_youtube_id("https://m.youtube.com/watch?v=xyz789"),
            Some("xyz789".into())
        );
    }

    #[test]
    fn youtube_no_id() {
        assert_eq!(extract_youtube_id("https://www.youtube.com/"), None);
    }

    #[test]
    fn valid_video_ids() {
        assert!(is_valid_video_id("dQw4w9WgXcQ"));
        assert!(is_valid_video_id("abc-_123"));
        assert!(!is_valid_video_id(""));
        assert!(!is_valid_video_id("../../etc"));
        assert!(!is_valid_video_id("id with spaces"));
    }

    #[test]
    fn is_youtube_domains() {
        assert!(is_youtube("www.youtube.com"));
        assert!(is_youtube("youtu.be"));
        assert!(is_youtube("m.youtube.com"));
        assert!(!is_youtube("www.github.com"));
    }
}
