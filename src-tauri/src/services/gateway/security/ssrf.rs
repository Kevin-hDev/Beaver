use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use tokio::net::lookup_host;
use url::Url;

const BLOCKED_METADATA_IPS: &[IpAddr] = &[
    IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254)),
    IpAddr::V4(Ipv4Addr::new(169, 254, 170, 2)),
    IpAddr::V4(Ipv4Addr::new(169, 254, 169, 253)),
    IpAddr::V4(Ipv4Addr::new(100, 100, 100, 200)),
    IpAddr::V6(Ipv6Addr::new(0xfd00, 0x0ec2, 0, 0, 0, 0, 0, 0x0254)),
];

const BLOCKED_METADATA_HOSTS: &[&str] = &["metadata.google.internal", "metadata.goog"];
const MAX_URL_CHARS: usize = 2048;
const BLOCKED_PORTS: &[u16] = &[
    0, 22, 23, 25, 53, 110, 135, 137, 138, 139, 143, 389, 445, 465, 587, 993, 995, 1433, 1521,
    2049, 2375, 2376, 3306, 3389, 5432, 5672, 5900, 6379, 9200, 9300, 11211, 27017,
];

pub struct SafeUrl {
    pub url: Url,
    pub host: String,
    pub port: u16,
    pub ip: IpAddr,
}

#[cfg(test)]
pub enum SsrfVerdict {
    Safe,
    Blocked(String),
}

#[cfg(test)]
pub async fn is_safe_url(url_str: &str, allow_private: bool) -> SsrfVerdict {
    match validate_url(url_str, allow_private).await {
        Ok(_) => SsrfVerdict::Safe,
        Err(reason) => SsrfVerdict::Blocked(reason),
    }
}

pub async fn validate_url(url_str: &str, allow_private: bool) -> Result<SafeUrl, String> {
    if url_str.chars().count() > MAX_URL_CHARS {
        return Err("URL trop longue".to_string());
    }
    let parsed = Url::parse(url_str).map_err(|_| "URL invalide".to_string())?;

    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("schéma non autorisé".to_string()),
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("identifiants URL interdits".to_string());
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "hôte manquant".to_string())?
        .to_lowercase()
        .trim_end_matches('.')
        .to_string();

    if BLOCKED_METADATA_HOSTS.iter().any(|h| host == *h) {
        return Err("cloud metadata bloqué".to_string());
    }
    if is_blocked_host_literal(&host) && !allow_private {
        return Err("adresse privée bloquée".to_string());
    }

    let port = parsed.port_or_known_default().unwrap_or(443);
    if is_blocked_port(port) {
        return Err("port non autorisé".to_string());
    }

    let lookup = format!("{}:{}", host, port);
    let addrs = resolve_host(&lookup).await?;
    let ip = validate_resolved_addrs(&addrs, allow_private)?;
    Ok(SafeUrl {
        url: parsed,
        host,
        port,
        ip,
    })
}

async fn resolve_host(lookup: &str) -> Result<Vec<std::net::SocketAddr>, String> {
    lookup_host(lookup)
        .await
        .map(|addrs| addrs.collect::<Vec<_>>())
        .map_err(|_| "résolution DNS échouée".to_string())
}

fn validate_resolved_addrs(
    addrs: &[std::net::SocketAddr],
    allow_private: bool,
) -> Result<IpAddr, String> {
    if addrs.is_empty() {
        return Err("aucune adresse résolue".to_string());
    }
    for addr in addrs {
        let ip = addr.ip();
        if is_metadata_ip(&ip) {
            return Err("cloud metadata bloqué".to_string());
        }
        if !allow_private && is_blocked_ip(&ip) {
            return Err("adresse privée bloquée".to_string());
        }
    }
    Ok(addrs[0].ip())
}

fn is_blocked_host_literal(host: &str) -> bool {
    host == "localhost"
        || host == "0.0.0.0"
        || host == "::1"
        || host.starts_with("0177.")
        || host.starts_with("0x7f")
        || host.starts_with("127.")
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || host.starts_with("169.254.")
        || host.starts_with("fc00:")
        || (host.starts_with("fd") && host.contains(':'))
        || host.starts_with("fe80:")
        || host.starts_with("::ffff:127.")
}

fn is_blocked_port(port: u16) -> bool {
    BLOCKED_PORTS.contains(&port)
}

pub fn is_metadata_ip(ip: &IpAddr) -> bool {
    BLOCKED_METADATA_IPS.contains(ip)
}

pub fn is_blocked_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_unspecified()
                || v4.is_multicast()
                || is_cgnat(v4)
                || is_reserved_v4(v4)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || is_ula(v6)
                || is_link_local_v6(v6)
                || is_mapped_private(v6)
        }
    }
}

fn is_cgnat(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 100 && (octets[1] & 0xC0) == 64
}

fn is_reserved_v4(ip: &Ipv4Addr) -> bool {
    let first = ip.octets()[0];
    first == 0 || first >= 240
}

fn is_ula(ip: &Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xfe00) == 0xfc00
}

fn is_link_local_v6(ip: &Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xffc0) == 0xfe80
}

fn is_mapped_private(ip: &Ipv6Addr) -> bool {
    matches!(ip.to_ipv4_mapped(), Some(v4) if is_blocked_ip(&IpAddr::V4(v4)))
}
