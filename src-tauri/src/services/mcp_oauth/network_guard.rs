use std::future::Future;
use std::net::SocketAddr;

use crate::services::gateway::security::ssrf::{is_blocked_ip, is_metadata_ip};

const MAX_DNS_ADDRESSES: usize = 32;

#[derive(Clone, Copy)]
pub enum DestinationRole {
    McpProbe,
    ResourceMetadata,
    OAuth,
    #[cfg(test)]
    TestLoopback,
}

pub struct CheckedDestination {
    pub url: reqwest::Url,
    pub address: SocketAddr,
}

pub async fn prepare_with<R, F>(
    connector_id: &str,
    mcp_endpoint: &str,
    role: DestinationRole,
    url: &str,
    resolver: R,
) -> Result<CheckedDestination, String>
where
    R: FnOnce(String, u16) -> F,
    F: Future<Output = Result<Vec<SocketAddr>, String>>,
{
    #[cfg(test)]
    let test_loopback = matches!(role, DestinationRole::TestLoopback);
    #[cfg(not(test))]
    let test_loopback = false;
    if url.len() > 2048 {
        return Err("destination OAuth non autorisée".to_string());
    }
    let parsed = reqwest::Url::parse(url).map_err(|_| "destination OAuth invalide")?;
    if (!test_loopback && parsed.scheme() != "https")
        || (!test_loopback && parsed.port_or_known_default() != Some(443))
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.fragment().is_some()
    {
        return Err("destination OAuth non autorisée".to_string());
    }
    let host = parsed.host_str().ok_or("destination OAuth invalide")?;
    #[cfg(test)]
    if test_loopback
        && (parsed.scheme() != "http"
            || parsed.port().is_none()
            || !parsed.host().is_some_and(|host| match host {
                url::Host::Ipv4(ip) => ip.is_loopback(),
                url::Host::Ipv6(ip) => ip.is_loopback(),
                url::Host::Domain(_) => false,
            })
            || url != mcp_endpoint)
    {
        return Err("destination OAuth non autorisée".to_string());
    }
    let port = parsed
        .port_or_known_default()
        .ok_or("destination OAuth invalide")?;
    let allowed = match role {
        DestinationRole::McpProbe => {
            crate::services::mcp_bridge::trusted::is_trusted_endpoint_for_connector(
                connector_id,
                url,
            )
        }
        DestinationRole::ResourceMetadata => {
            crate::services::mcp_bridge::trusted::is_trusted_endpoint_for_connector(
                connector_id,
                mcp_endpoint,
            ) && (reqwest::Url::parse(mcp_endpoint)
                .ok()
                .is_some_and(|endpoint| endpoint.origin() == parsed.origin())
                || super::trusted_oauth::validate_endpoint(connector_id, url).is_ok())
        }
        DestinationRole::OAuth => {
            super::trusted_oauth::validate_endpoint(connector_id, url).is_ok()
        }
        #[cfg(test)]
        DestinationRole::TestLoopback => true,
    };
    if !allowed {
        return Err("destination OAuth non autorisée".to_string());
    }
    let addresses = resolver(host.to_string(), port).await?;
    if addresses.is_empty() || addresses.len() > MAX_DNS_ADDRESSES {
        return Err("résolution OAuth refusée".to_string());
    }
    if addresses.iter().any(|address| {
        address.port() != port
            || is_metadata_ip(&address.ip())
            || (test_loopback && !address.ip().is_loopback())
            || (!test_loopback && is_blocked_ip(&address.ip()))
    }) {
        return Err("résolution OAuth refusée".to_string());
    }
    let address = addresses[0];
    Ok(CheckedDestination {
        url: parsed,
        address,
    })
}

#[cfg(test)]
pub async fn destination_loopback_for_test(
    connector_id: &str,
    expected_url: &str,
    url: &str,
) -> Result<crate::services::secure_http_destination::FixedDestination, String> {
    let parsed = reqwest::Url::parse(url).map_err(|_| "destination OAuth invalide")?;
    let ip = match parsed.host() {
        Some(url::Host::Ipv4(ip)) => std::net::IpAddr::V4(ip),
        Some(url::Host::Ipv6(ip)) => std::net::IpAddr::V6(ip),
        _ => return Err("destination OAuth non autorisée".to_string()),
    };
    let port = parsed.port().ok_or("destination OAuth invalide")?;
    let checked = prepare_with(
        connector_id,
        expected_url,
        DestinationRole::TestLoopback,
        url,
        |_, _| async move { Ok(vec![SocketAddr::new(ip, port)]) },
    )
    .await?;
    crate::services::secure_http_destination::FixedDestination::new_loopback(
        checked.url,
        std::time::Duration::from_secs(2),
    )
    .map_err(|_| "destination OAuth indisponible".to_string())
}

pub async fn prepare(
    connector_id: &str,
    mcp_endpoint: &str,
    role: DestinationRole,
    url: &str,
) -> Result<CheckedDestination, String> {
    tokio::time::timeout(
        std::time::Duration::from_secs(15),
        prepare_with(
            connector_id,
            mcp_endpoint,
            role,
            url,
            |host, port| async move {
                let addrs = tokio::net::lookup_host((host.as_str(), port))
                    .await
                    .map_err(|_| "résolution OAuth refusée".to_string())?;
                Ok(addrs.take(MAX_DNS_ADDRESSES + 1).collect())
            },
        ),
    )
    .await
    .map_err(|_| "résolution OAuth expirée".to_string())?
}

pub async fn destination(
    connector_id: &str,
    mcp_endpoint: &str,
    role: DestinationRole,
    url: &str,
) -> Result<crate::services::secure_http_destination::FixedDestination, String> {
    let checked = prepare(connector_id, mcp_endpoint, role, url).await?;
    crate::services::secure_http_destination::FixedDestination::new(
        checked.url,
        checked.address,
        std::time::Duration::from_secs(15),
    )
    .map_err(|_| "destination OAuth indisponible".to_string())
}

#[cfg(test)]
#[path = "network_guard_tests.rs"]
mod tests;
