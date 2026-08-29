//! Vue publique du catalogue, dérivée de l'autorité privée `route_profile`.

use super::route_profile::{self, CatalogPolicy};

#[derive(Debug, Clone, Copy)]
pub struct ProviderSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub base_url: &'static str,
    pub models_endpoint: &'static str,
    pub signup_url: &'static str,
}

pub fn all() -> Vec<ProviderSpec> {
    route_profile::public_api()
        .filter_map(to_public_spec)
        .collect()
}

pub fn find(provider_id: &str) -> Option<ProviderSpec> {
    to_public_spec(route_profile::find(provider_id)?)
}

fn to_public_spec(profile: &'static route_profile::RouteProfile) -> Option<ProviderSpec> {
    let CatalogPolicy::PublicApi { signup_url } = profile.catalog else {
        return None;
    };
    let (base_url, models_endpoint) = profile.endpoint.static_parts()?;
    Some(ProviderSpec {
        id: profile.id.provider_id(),
        display_name: profile.display_name,
        base_url,
        models_endpoint,
        signup_url,
    })
}
