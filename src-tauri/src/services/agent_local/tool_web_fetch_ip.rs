use std::net::IpAddr;

pub fn is_172_private(host: &str) -> bool {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    if let Ok(first) = parts[0].parse::<u8>() {
        if first == 172 {
            if let Ok(second) = parts[1].parse::<u8>() {
                return (16..=31).contains(&second);
            }
        }
    }
    false
}

pub fn is_ip_private(ip: &IpAddr) -> bool {
    crate::services::gateway::security::ssrf::is_blocked_ip(ip)
}
