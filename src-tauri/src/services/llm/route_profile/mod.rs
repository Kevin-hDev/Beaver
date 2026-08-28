mod catalog;
mod catalog_api;
mod catalog_local;
mod catalog_oauth;
mod policies;
mod policy_types;
mod tool_policies;
mod types;

#[cfg(test)]
pub(super) use catalog::{all, find_id};
pub(super) use catalog::{find, public_api};
pub(crate) use policy_types::{ExtensionToolPolicy, ResolvedToolPolicy, SchemaPolicy};
pub(super) use types::*;

pub(crate) fn tool_policy(provider_id: &str, model: &str) -> Option<ResolvedToolPolicy> {
    let profile = find(provider_id)?;
    Some(tool_policies::resolve(profile, model))
}

#[cfg(test)]
mod tests;
