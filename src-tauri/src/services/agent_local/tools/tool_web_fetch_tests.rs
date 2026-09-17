#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_web_fetch::validate_url;
    use crate::services::agent_local::tool_web_fetch_ip::{is_172_private, is_ip_private};

    // --- is_172_private ---

    #[test]
    fn is_172_private_blocks_172_16() {
        assert!(is_172_private("172.16.0.1"));
        assert!(is_172_private("172.31.255.255"));
    }

    #[test]
    fn is_172_private_allows_172_32() {
        assert!(!is_172_private("172.32.0.1"));
        assert!(!is_172_private("172.15.0.1"));
    }

    // --- is_ip_private ---

    #[test]
    fn is_ip_private_v4_loopback() {
        use std::net::{IpAddr, Ipv4Addr};
        assert!(is_ip_private(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
    }

    #[test]
    fn is_ip_private_v4_private() {
        use std::net::{IpAddr, Ipv4Addr};
        assert!(is_ip_private(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
        assert!(is_ip_private(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(is_ip_private(&IpAddr::V4(Ipv4Addr::new(172, 20, 0, 1))));
    }

    #[test]
    fn is_ip_private_v4_public() {
        use std::net::{IpAddr, Ipv4Addr};
        assert!(!is_ip_private(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
        assert!(!is_ip_private(&IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))));
    }

    #[test]
    fn is_ip_private_v6_loopback() {
        use std::net::{IpAddr, Ipv6Addr};
        assert!(is_ip_private(&IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }

    #[test]
    fn is_ip_private_v6_ula() {
        use std::net::{IpAddr, Ipv6Addr};
        let ula: Ipv6Addr = "fc00::1".parse().unwrap();
        assert!(is_ip_private(&IpAddr::V6(ula)));
        let ula2: Ipv6Addr = "fd12:3456:789a:1::1".parse().unwrap();
        assert!(is_ip_private(&IpAddr::V6(ula2)));
    }

    #[test]
    fn is_ip_private_v6_link_local() {
        use std::net::{IpAddr, Ipv6Addr};
        let ll: Ipv6Addr = "fe80::1".parse().unwrap();
        assert!(is_ip_private(&IpAddr::V6(ll)));
    }

    #[test]
    fn is_ip_private_v6_mapped_loopback() {
        use std::net::{IpAddr, Ipv6Addr};
        let mapped: Ipv6Addr = "::ffff:127.0.0.1".parse().unwrap();
        assert!(is_ip_private(&IpAddr::V6(mapped)));
    }

    // --- validate_url (blocklist statique, sans DNS) ---

    #[tokio::test]
    async fn validate_url_blocks_localhost() {
        assert!(validate_url("http://localhost/").await.is_err());
        assert!(validate_url("http://127.0.0.1/").await.is_err());
    }

    #[tokio::test]
    async fn validate_url_blocks_ipv6_loopback() {
        assert!(validate_url("http://[::1]/").await.is_err());
    }

    #[tokio::test]
    async fn validate_url_blocks_octal_loopback() {
        assert!(validate_url("http://0177.0.0.1/").await.is_err());
    }

    #[tokio::test]
    async fn validate_url_blocks_172_private() {
        assert!(validate_url("http://172.20.0.1/").await.is_err());
    }

    #[tokio::test]
    async fn validate_url_rejects_non_http() {
        assert!(validate_url("ftp://example.com").await.is_err());
        assert!(validate_url("file:///etc/passwd").await.is_err());
    }
}
