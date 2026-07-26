use crate::services::agent_local::tool_web_fetch_ip::{is_172_private, is_ip_private};
use std::net::IpAddr;

const MAX_URL_LEN: usize = 2048;

pub async fn validate_url(url: &str) -> Result<(), String> {
    validate_url_with_ip(url).await.map(|_| ())
}

pub async fn validate_url_with_ip(url: &str) -> Result<(String, std::net::IpAddr), String> {
    if url.len() > MAX_URL_LEN {
        return Err("Preview unavailable".into());
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("Preview unavailable".into());
    }
    let host = extract_host(url);
    if host.is_empty() {
        return Err("Preview unavailable".into());
    }
    if is_blocked_host(&host) {
        return Err("Preview unavailable".into());
    }
    let ip = verify_dns_with_ip(&host).await?;
    Ok((host, ip))
}

pub fn is_safe_resource_url(url: &str) -> bool {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return false;
    }
    let host = extract_host(url);
    !host.is_empty() && !is_blocked_host(&host)
}

fn extract_host(url: &str) -> String {
    let authority = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("");
    let host = if authority.starts_with('[') {
        authority
            .split(']')
            .next()
            .unwrap_or("")
            .trim_start_matches('[')
    } else {
        authority.split(':').next().unwrap_or("")
    };
    host.to_string()
}

fn is_blocked_host(host: &str) -> bool {
    host == "localhost"
        || host == "0.0.0.0"
        || host == "::1"
        || host == "[::1]"
        || host.starts_with("0177.")
        || host.starts_with("0x7f")
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || host.starts_with("127.")
        || is_172_private(host)
        || host.starts_with("169.254.")
        || host.starts_with("fc00:")
        || (host.starts_with("fd") && host.contains(':'))
        || host.starts_with("fe80:")
        || host.starts_with("::ffff:127.")
}

async fn verify_dns_with_ip(host: &str) -> Result<IpAddr, String> {
    let lookup = format!("{host}:80");
    let addrs: Vec<_> = tokio::net::lookup_host(&lookup)
        .await
        .map_err(|_| "Preview unavailable".to_string())?
        .collect();
    for addr in &addrs {
        if is_ip_private(&addr.ip()) {
            return Err("Preview unavailable".into());
        }
    }
    addrs
        .first()
        .map(|a| a.ip())
        .ok_or_else(|| "Preview unavailable".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_hosts() {
        assert!(is_blocked_host("localhost"));
        assert!(is_blocked_host("127.0.0.1"));
        assert!(is_blocked_host("10.0.0.1"));
        assert!(is_blocked_host("192.168.1.1"));
        assert!(is_blocked_host("169.254.0.1"));
        assert!(!is_blocked_host("github.com"));
    }

    #[test]
    fn url_too_long() {
        let long = format!("https://example.com/{}", "a".repeat(3000));
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(validate_url(&long));
        assert!(result.is_err());
    }

    #[test]
    fn safe_resource_urls() {
        assert!(is_safe_resource_url("https://img.com/pic.jpg"));
        assert!(is_safe_resource_url("http://img.com/pic.jpg"));
        assert!(!is_safe_resource_url("javascript:alert(1)"));
        assert!(!is_safe_resource_url("data:text/html,<script>"));
        assert!(!is_safe_resource_url("blob:http://evil.com/x"));
    }

    #[test]
    fn resource_url_blocks_private_hosts() {
        assert!(!is_safe_resource_url("http://192.168.1.1/logo.png"));
        assert!(!is_safe_resource_url("http://127.0.0.1/favicon.ico"));
        assert!(!is_safe_resource_url("http://10.0.0.1/img.jpg"));
        assert!(!is_safe_resource_url("http://localhost/pic.png"));
        assert!(is_safe_resource_url("https://cdn.github.com/img.png"));
    }
}
