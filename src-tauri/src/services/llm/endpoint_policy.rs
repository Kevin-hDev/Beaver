#![allow(
    dead_code,
    reason = "configurable endpoint variants are compiled before candidate route activation"
)]

use super::route_profile::EndpointPolicy;

pub(super) async fn resolve(
    policy: EndpointPolicy,
    supplied: Option<&str>,
) -> Result<String, &'static str> {
    match policy {
        EndpointPolicy::Static { base_url, .. } => no_override(base_url, supplied),
        EndpointPolicy::PinnedBackend { base_url } => no_override(base_url, supplied),
        EndpointPolicy::RegionAllowlist { regions } => {
            let selected = supplied.ok_or("provider_configuration_invalid")?;
            regions
                .iter()
                .find_map(|(region, url)| (*region == selected).then_some((*url).to_string()))
                .ok_or("provider_configuration_invalid")
        }
        EndpointPolicy::Workspace { host_suffix } => {
            let workspace = supplied.ok_or("provider_configuration_invalid")?;
            if crate::services::provider_connections::workspace_id::validate(workspace).is_err() {
                return Err("provider_configuration_invalid");
            }
            Ok(format!("https://{workspace}.{host_suffix}"))
        }
        EndpointPolicy::ValidatedHttps => validated_https(supplied).await,
        EndpointPolicy::ConnectionConfigured | EndpointPolicy::OllamaLocal => {
            Err("provider_configuration_invalid")
        }
    }
}

fn no_override(base_url: &str, supplied: Option<&str>) -> Result<String, &'static str> {
    if supplied.is_some_and(|value| value != base_url) {
        return Err("provider_configuration_invalid");
    }
    Ok(base_url.to_string())
}

async fn validated_https(supplied: Option<&str>) -> Result<String, &'static str> {
    let raw = supplied.ok_or("provider_configuration_invalid")?;
    let parsed = url::Url::parse(raw).map_err(|_| "provider_configuration_invalid")?;
    if parsed.scheme() != "https"
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err("provider_configuration_invalid");
    }
    crate::services::gateway::security::ssrf::validate_url(raw, false)
        .await
        .map_err(|_| "provider_configuration_invalid")?;
    Ok(parsed.to_string().trim_end_matches('/').to_string())
}
