use crate::services::{
    brand,
    gateway::security::ssrf::{self, SafeUrl},
};
use futures_util::StreamExt;
use reqwest::{header::CONTENT_TYPE, Client, StatusCode};
use std::net::SocketAddr;
use std::time::Duration;
use url::Url;

const MAX_OUTPUT_CHARS: usize = 50_000;
const MAX_BODY_BYTES: usize = 5 * 1024 * 1024;
const MAX_REDIRECTS: usize = 3;
const TIMEOUT: Duration = Duration::from_secs(15);
const MIN_READABLE_LEN: usize = 100;

pub async fn fetch_url(url: &str) -> Result<String, String> {
    fetch_url_checked(url, false).await
}

async fn fetch_url_checked(url: &str, allow_private: bool) -> Result<String, String> {
    let mut current = url.to_string();
    for redirect_count in 0..=MAX_REDIRECTS {
        let safe = ssrf::validate_url(&current, allow_private).await?;
        let client = pinned_client(&safe)?;
        let resp = client
            .get(safe.url.clone())
            .header("User-Agent", format!("{} WebFetch", brand::user_agent()))
            .timeout(TIMEOUT)
            .send()
            .await
            .map_err(|e| format!("Erreur fetch: {}", clean_reqwest_error(&e)))?;

        if resp.status().is_redirection() {
            if redirect_count == MAX_REDIRECTS {
                return Err("Trop de redirections".to_string());
            }
            current = redirect_target(&safe.url, resp.headers())?;
            continue;
        }
        return finish_response(resp, &safe.url).await;
    }
    Err("Trop de redirections".to_string())
}

fn pinned_client(safe: &SafeUrl) -> Result<Client, String> {
    Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .resolve(&safe.host, SocketAddr::new(safe.ip, safe.port))
        .build()
        .map_err(|_| "Erreur client".to_string())
}

async fn finish_response(resp: reqwest::Response, url: &Url) -> Result<String, String> {
    let status = resp.status();
    if !status.is_success() {
        return Err(format_http_error(status));
    }
    let content_type = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    if !is_supported_content_type(content_type.as_deref()) {
        return Err("Type de contenu non supporté".to_string());
    }
    let body = read_body_bounded(resp).await?;
    let decoded = String::from_utf8_lossy(&body);
    let content = render_content(&decoded, content_type.as_deref(), url);
    Ok(truncate_chars(&content, MAX_OUTPUT_CHARS))
}

fn redirect_target(base: &Url, headers: &reqwest::header::HeaderMap) -> Result<String, String> {
    let location = headers
        .get(reqwest::header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| "Redirection invalide".to_string())?;
    base.join(location)
        .map(|url| url.to_string())
        .map_err(|_| "Redirection invalide".to_string())
}

async fn read_body_bounded(resp: reqwest::Response) -> Result<Vec<u8>, String> {
    let mut body = Vec::new();
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Erreur lecture: {}", clean_reqwest_error(&e)))?;
        if body.len().saturating_add(chunk.len()) > MAX_BODY_BYTES {
            return Err("Réponse trop volumineuse".to_string());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn render_content(body: &str, content_type: Option<&str>, url: &Url) -> String {
    if is_html(content_type, body) {
        extract_and_convert(body, url.as_str())
    } else {
        body.to_string()
    }
}

fn is_supported_content_type(content_type: Option<&str>) -> bool {
    let Some(value) = content_type.map(|v| v.to_ascii_lowercase()) else {
        return true;
    };
    value.starts_with("text/")
        || value.starts_with("application/json")
        || value.starts_with("application/xhtml+xml")
        || value.starts_with("application/xml")
        || value.starts_with("application/ld+json")
        || value.contains("+json")
}

fn is_html(content_type: Option<&str>, body: &str) -> bool {
    content_type
        .map(|v| {
            let lower = v.to_ascii_lowercase();
            lower.starts_with("text/html") || lower.starts_with("application/xhtml+xml")
        })
        .unwrap_or_else(|| {
            body.trim_start().starts_with("<!doctype html") || body.contains("<html")
        })
}

fn extract_and_convert(html: &str, url: &str) -> String {
    let readability_result = extract_readability(html, url);
    if readability_result.len() > MIN_READABLE_LEN {
        return readability_result;
    }
    convert_to_markdown(html)
}

fn extract_readability(html: &str, url: &str) -> String {
    use dom_smoothie::{Config, Readability};
    Readability::new(html, Some(url), Some(Config::default()))
        .ok()
        .and_then(|mut r| r.parse().ok())
        .map(|a| a.text_content.to_string())
        .unwrap_or_default()
}

fn convert_to_markdown(html: &str) -> String {
    use htmd::HtmlToMarkdown;
    HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style"])
        .build()
        .convert(html)
        .unwrap_or_default()
}

fn format_http_error(status: StatusCode) -> String {
    format!("HTTP {}", status.as_u16())
}

fn clean_reqwest_error(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        return "timeout".to_string();
    }
    if error.is_connect() {
        return "connexion impossible".to_string();
    }
    "requête échouée".to_string()
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    let mut out: String = input.chars().take(max_chars).collect();
    out.push_str("... [tronqué]");
    out
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) async fn validate_url(url: &str) -> Result<(), String> {
    ssrf::validate_url(url, false).await.map(|_| ())
}

#[cfg(test)]
pub(crate) async fn fetch_url_allow_private(url: &str) -> Result<String, String> {
    fetch_url_checked(url, true).await
}
