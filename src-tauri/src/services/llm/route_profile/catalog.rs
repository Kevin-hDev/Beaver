use super::catalog_api::API_PROFILES;
use super::catalog_local::LOCAL_PROFILES;
use super::catalog_oauth::OAUTH_PROFILES;
use super::types::RouteProfile;
use crate::services::reasoning_continuity::contract::RouteId;

pub(in crate::services::llm) fn all() -> impl Iterator<Item = &'static RouteProfile> {
    API_PROFILES
        .iter()
        .chain(OAUTH_PROFILES)
        .chain(LOCAL_PROFILES)
}

pub(in crate::services::llm) fn public_api() -> impl Iterator<Item = &'static RouteProfile> {
    API_PROFILES.iter()
}

pub(in crate::services::llm) fn find(provider_id: &str) -> Option<&'static RouteProfile> {
    let id = RouteId::from_provider_id(provider_id)?;
    find_id(id)
}

pub(in crate::services::llm) fn find_id(id: RouteId) -> Option<&'static RouteProfile> {
    all().find(|profile| profile.id == id)
}
