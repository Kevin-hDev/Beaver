use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::ssrf::*;

#[test]
fn metadata_ips_blocked() {
    assert!(is_metadata_ip(&IpAddr::V4(Ipv4Addr::new(
        169, 254, 169, 254
    ))));
    assert!(is_metadata_ip(&IpAddr::V4(Ipv4Addr::new(169, 254, 170, 2))));
    assert!(is_metadata_ip(&IpAddr::V4(Ipv4Addr::new(
        100, 100, 100, 200
    ))));
    assert!(!is_metadata_ip(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
}

#[test]
fn cgnat_blocked() {
    assert!(is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(100, 64, 0, 1))));
    assert!(is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(
        100, 127, 255, 254
    ))));
    assert!(!is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(100, 128, 0, 1))));
    assert!(!is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(
        100, 63, 255, 254
    ))));
}

#[test]
fn private_ips_blocked() {
    assert!(is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    assert!(is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))));
    assert!(is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
    assert!(is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
    assert!(is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))));
}

#[test]
fn public_ips_allowed() {
    assert!(!is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
    assert!(!is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))));
    assert!(!is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34))));
}

#[test]
fn reserved_v4_blocked() {
    assert!(is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(0, 0, 0, 1))));
    assert!(is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(240, 0, 0, 1))));
    assert!(is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(
        255, 255, 255, 255
    ))));
    assert!(!is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1))));
}

#[test]
fn ipv6_ula_blocked() {
    let ula = Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1);
    assert!(is_blocked_ip(&IpAddr::V6(ula)));
    let fd = Ipv6Addr::new(0xfd12, 0x3456, 0, 0, 0, 0, 0, 1);
    assert!(is_blocked_ip(&IpAddr::V6(fd)));
}

#[test]
fn ipv6_link_local_blocked() {
    let ll = Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1);
    assert!(is_blocked_ip(&IpAddr::V6(ll)));
}

#[test]
fn multicast_blocked() {
    assert!(is_blocked_ip(&IpAddr::V4(Ipv4Addr::new(224, 0, 0, 1))));
    let mc6 = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 1);
    assert!(is_blocked_ip(&IpAddr::V6(mc6)));
}

#[tokio::test]
async fn rejects_non_http_scheme() {
    match is_safe_url("ftp://example.com", false).await {
        SsrfVerdict::Blocked(reason) => assert!(reason.contains("schéma")),
        SsrfVerdict::Safe => panic!("ftp must be blocked"),
    }
}

#[tokio::test]
async fn rejects_metadata_host() {
    assert!(matches!(
        is_safe_url("http://metadata.google.internal/computeMetadata/v1", true).await,
        SsrfVerdict::Blocked(_)
    ));
}

#[tokio::test]
async fn rejects_missing_host() {
    assert!(matches!(
        is_safe_url("http:///path", false).await,
        SsrfVerdict::Blocked(_)
    ));
}

#[tokio::test]
async fn rejects_url_credentials() {
    assert!(matches!(
        is_safe_url("https://user:pass@example.com", false).await,
        SsrfVerdict::Blocked(_)
    ));
}

#[tokio::test]
async fn rejects_blocked_ports_before_dns() {
    assert!(matches!(
        is_safe_url("https://example.com:22", false).await,
        SsrfVerdict::Blocked(_)
    ));
}
