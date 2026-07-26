pub mod parse;
pub mod providers;
pub mod security;

use futures_util::StreamExt;
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

use crate::services::brand;

pub const TIMEOUT: Duration = Duration::from_secs(5);
const MAX_HTML: usize = 100_000;
const MAX_REDIRECTS: usize = 3;

#[derive(Debug, Clone, Serialize)]
pub struct LinkPreview {
    pub url: String,
    pub domain: String,
    pub site_name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub favicon: Option<String>,
}

pub async fn fetch_preview(url: &str) -> Result<LinkPreview, String> {
    let (host, ip) = security::validate_url_with_ip(url).await?;
    let domain = parse::extract_domain(url);

    if providers::is_youtube(&domain) {
        if let Ok(preview) = providers::youtube_preview(url, &domain).await {
            return Ok(preview);
        }
    }

    let html = fetch_html_pinned(url, &host, ip).await?;
    let base = format!(
        "{}://{}",
        if url.starts_with("https") {
            "https"
        } else {
            "http"
        },
        domain
    );

    let site_name = parse::extract_og(&html, "og:site_name");
    let title = parse::extract_og(&html, "og:title").or_else(|| parse::extract_tag(&html, "title"));
    let description = parse::extract_og(&html, "og:description")
        .or_else(|| parse::extract_meta_name(&html, "description"));
    let image = parse::extract_og(&html, "og:image")
        .map(|img| parse::resolve_url(&img, &base))
        .filter(|u| security::is_safe_resource_url(u));
    let favicon = parse::extract_favicon(&html)
        .map(|f| parse::resolve_url(&f, &base))
        .filter(|u| security::is_safe_resource_url(u))
        .or_else(|| Some(format!("{}/favicon.ico", base)));

    Ok(LinkPreview {
        url: url.to_string(),
        domain,
        site_name,
        title,
        description,
        image,
        favicon,
    })
}

async fn fetch_html_pinned(url: &str, host: &str, ip: std::net::IpAddr) -> Result<String, String> {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .resolve(host, std::net::SocketAddr::new(ip, 0))
        .build()
        .map_err(|_| "Preview unavailable".to_string())?;

    let mut current_url = url.to_string();
    for _ in 0..=MAX_REDIRECTS {
        let resp = client
            .get(&current_url)
            .header("User-Agent", format!("{} LinkPreview", brand::user_agent()))
            .timeout(TIMEOUT)
            .send()
            .await
            .map_err(|_| "Preview unavailable".to_string())?;

        if resp.status().is_redirection() {
            let location = resp
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .ok_or("Preview unavailable")?;
            let next = resolve_redirect(&current_url, location);
            security::validate_url(&next).await?;
            current_url = next;
            continue;
        }

        if !resp.status().is_success() {
            return Err("Preview unavailable".into());
        }
        return read_body_bounded(resp).await;
    }
    Err("Preview unavailable".into())
}

fn resolve_redirect(base: &str, location: &str) -> String {
    if location.starts_with("http://") || location.starts_with("https://") {
        location.to_string()
    } else if location.starts_with('/') {
        let origin = base.split('/').take(3).collect::<Vec<_>>().join("/");
        format!("{origin}{location}")
    } else {
        location.to_string()
    }
}

async fn read_body_bounded(resp: reqwest::Response) -> Result<String, String> {
    let mut body = Vec::with_capacity(MAX_HTML);
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| "Preview unavailable".to_string())?;
        let remaining = MAX_HTML.saturating_sub(body.len());
        if remaining == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
    }
    Ok(String::from_utf8_lossy(&body).into_owned())
}
