mod cache_policies;
mod catalog;
mod catalog_api;
mod catalog_local;
mod catalog_oauth;
mod payload_policies;
mod policies;
mod policy_types;
mod tool_policies;
mod types;

#[cfg(test)]
pub(super) use catalog::{all, find_id};
pub(super) use catalog::{find, public_api};
pub(crate) use policy_types::{
    CachePolicy, ExtensionToolPolicy, ResolvedCachePolicy, ResolvedPayloadPolicy,
    ResolvedToolPolicy, SchemaPolicy,
};
pub(super) use types::*;
pub(crate) use types::{ImageFormat, MessageWirePolicy, ToolResultPlacement};

pub(crate) fn tool_policy(provider_id: &str, model: &str) -> Option<ResolvedToolPolicy> {
    let profile = find(provider_id)?;
    Some(tool_policies::resolve(profile, model))
}

pub(crate) fn cache_policy<'a>(
    provider_id: &str,
    model: &'a str,
) -> Option<ResolvedCachePolicy<'a>> {
    let profile = find(provider_id)?;
    Some(cache_policies::resolve(profile, model))
}

pub(crate) fn payload_policy(provider_id: &str, model: &str) -> Option<ResolvedPayloadPolicy> {
    let profile = find(provider_id)?;
    Some(payload_policies::resolve(profile, model))
}

#[cfg(test)]
pub(super) fn anthropic_fixture(
    max_tokens: Option<u32>,
) -> Result<serde_json::Value, &'static str> {
    payload_policies::anthropic_fixture(max_tokens)
}

#[cfg(test)]
mod tests;
