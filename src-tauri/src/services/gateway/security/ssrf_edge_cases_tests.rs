//! Edge cases SSRF : bypass literals (octal/hex/localhost), ports bloqués,
//! URLs trop longues, schémas exotiques, IPv4-mapped IPv6, metadata même avec
//! allow_private=true.
//!
//! Les fonctions privées (is_blocked_host_literal, is_blocked_port, is_cgnat,
//! is_reserved_v4, is_ula, is_link_local_v6, is_mapped_private) sont couvertes
//! indirectement via les helpers publics is_blocked_ip + is_safe_url.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::ssrf::*;

// --- Bypass literals bloqués AVANT le DNS -----------------------------------

#[tokio::test]
async fn blocks_octal_loopback_literal() {
    // 0177.0.0.1 = 127.0.0.1 en octal. Doit être bloqué par le literal check,
    // pas attendre la résolution DNS (sinon DNS rebinding possible).
    assert!(matches!(
        is_safe_url("http://0177.0.0.1/", false).await,
        SsrfVerdict::Blocked(_)
    ));
}

#[tokio::test]
async fn blocks_hex_loopback_literal() {
    assert!(matches!(
        is_safe_url("http://0x7f.0.0.1/", false).await,
        SsrfVerdict::Blocked(_)
    ));
}

#[tokio::test]
async fn blocks_localhost_literal() {
    assert!(matches!(
        is_safe_url("http://localhost/", false).await,
        SsrfVerdict::Blocked(_)
    ));
}

#[tokio::test]
async fn blocks_zero_zero_zero_zero_literal() {
    assert!(matches!(
        is_safe_url("http://0.0.0.0/", false).await,
        SsrfVerdict::Blocked(_)
    ));
}

#[tokio::test]
async fn blocks_ipv6_loopback_literal() {
    assert!(matches!(
        is_safe_url("http://[::1]/", false).await,
        SsrfVerdict::Blocked(_)
    ));
}

// --- validate_url : longueur, schémas, ports --------------------------------

#[tokio::test]
async fn blocks_oversized_url() {
    let huge = format!("http://example.com/{}", "a".repeat(2100));
    assert!(matches!(
        is_safe_url(&huge, false).await,
        SsrfVerdict::Blocked(_)
    ));
}

#[tokio::test]
async fn blocks_file_scheme() {
    assert!(matches!(
        is_safe_url("file:///etc/passwd", false).await,
        SsrfVerdict::Blocked(_)
    ));
}

#[tokio::test]
async fn blocks_gopher_scheme() {
    assert!(matches!(
        is_safe_url("gopher://example.com/", false).await,
        SsrfVerdict::Blocked(_)
    ));
}

#[tokio::test]
async fn blocks_smtp_port() {
    assert!(matches!(
        is_safe_url("https://example.com:25/", false).await,
        SsrfVerdict::Blocked(_)
    ));
}

#[tokio::test]
async fn blocks_mysql_port() {
    assert!(matches!(
        is_safe_url("https://example.com:3306/", false).await,
        SsrfVerdict::Blocked(_)
    ));
}

#[tokio::test]
async fn blocks_redis_port() {
    assert!(matches!(
        is_safe_url("https://example.com:6379/", false).await,
        SsrfVerdict::Blocked(_)
    ));
}

// --- Metadata IP bloquée MÊME avec allow_private=true -----------------------

#[tokio::test]
async fn metadata_always_blocked_even_with_allow_private() {
    // allow_private=true autorise les IP privées, MAIS les metadata doivent
    // toujours être bloquées (sinon fuite de credentials cloud).
    assert!(matches!(
        is_safe_url("http://metadata.google.internal/computeMetadata/v1", true).await,
        SsrfVerdict::Blocked(_)
    ));
}

// --- IPv4-mapped IPv6 privées ------------------------------------------------

#[test]
fn mapped_ipv4_loopback_blocked() {
    // ::ffff:127.0.0.1 = 127.0.0.1 encapsulé en IPv6. Bypass classique.
    let mapped = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x7f00, 0x0001);
    assert!(is_blocked_ip(&IpAddr::V6(mapped)));
}

#[test]
fn mapped_ipv4_private_blocked() {
    // ::ffff:192.168.1.1
    let mapped = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc0a8, 0x0101);
    assert!(is_blocked_ip(&IpAddr::V6(mapped)));
}

#[test]
fn mapped_public_ipv4_not_blocked() {
    // ::ffff:8.8.8.8 = adresse publique → ne doit PAS être bloquée.
    let mapped = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x0808, 0x0808);
    assert!(!is_blocked_ip(&IpAddr::V6(mapped)));
}

// --- Bornes exactes CGNAT / reserved ----------------------------------------

#[test]
fn cgnat_lower_boundary_blocked() {
    // 100.64.0.0 = début exact du range CGNAT.
    assert!(is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(100, 64, 0, 0))));
}

#[test]
fn cgnat_upper_boundary_blocked() {
    // 100.127.255.255 = fin exacte du range CGNAT.
    assert!(is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(
        100, 127, 255, 255
    ))));
}

#[test]
fn just_below_cgnat_not_blocked() {
    // 100.63.255.255 = juste avant le range.
    assert!(!is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(
        100, 63, 255, 255
    ))));
}

#[test]
fn just_above_cgnat_not_blocked() {
    // 100.128.0.0 = juste après le range.
    assert!(!is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(100, 128, 0, 0))));
}
