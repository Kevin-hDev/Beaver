//! Vue publique du catalogue, dérivée de l'autorité privée `route_profile`.

use super::route_profile::{self, CatalogPolicy};
use crate::models::provider_contract::ProviderConnectionKind;

#[derive(Debug, Clone, Copy)]
pub struct ProviderSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub base_url: Option<&'static str>,
    pub models_endpoint: Option<&'static str>,
    pub signup_url: &'static str,
    pub connection_kind: ProviderConnectionKind,
}

pub fn all() -> Vec<ProviderSpec> {
    route_profile::public_api()
        .filter_map(to_public_spec)
        .collect()
}

#[allow(dead_code, reason = "kept as the public-only catalog lookup authority")]
pub fn find(provider_id: &str) -> Option<ProviderSpec> {
    let profile = route_profile::find(provider_id)?;
    matches!(profile.catalog, CatalogPolicy::PublicApi { .. }).then_some(())?;
    to_spec(profile)
}

#[allow(dead_code, reason = "consumed by the candidate settings UI in task 6")]
pub fn configurable() -> Vec<ProviderSpec> {
    route_profile::configurable().filter_map(to_spec).collect()
}

pub fn find_configurable(provider_id: &str) -> Option<ProviderSpec> {
    let profile = route_profile::find(provider_id)?;
    matches!(
        profile.catalog,
        CatalogPolicy::PublicApi { .. } | CatalogPolicy::ConfigurableApi { .. }
    )
    .then_some(())?;
    to_spec(profile)
}

fn to_public_spec(profile: &'static route_profile::RouteProfile) -> Option<ProviderSpec> {
    if !matches!(profile.catalog, CatalogPolicy::PublicApi { .. }) {
        return None;
    }
    to_spec(profile)
}

fn to_spec(profile: &'static route_profile::RouteProfile) -> Option<ProviderSpec> {
    let signup_url = match profile.catalog {
        CatalogPolicy::PublicApi { signup_url } | CatalogPolicy::ConfigurableApi { signup_url } => {
            signup_url
        }
        CatalogPolicy::Hidden => unreachable!("hidden routes are filtered before conversion"),
    };
    let (base_url, models_endpoint) = profile
        .endpoint
        .static_parts()
        .map_or((None, None), |(base, models)| (Some(base), Some(models)));
    Some(ProviderSpec {
        id: profile.id.provider_id(),
        display_name: profile.display_name,
        base_url,
        models_endpoint,
        signup_url,
        connection_kind: if matches!(
            profile.endpoint,
            route_profile::EndpointPolicy::ProviderConnection {
                resolver: route_profile::ConnectionEndpointResolver::QwenModelStudio
            }
        ) {
            ProviderConnectionKind::QwenModelStudio
        } else {
            ProviderConnectionKind::ApiKey
        },
    })
}
