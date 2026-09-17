use std::collections::BTreeSet;

use super::types::{self, OptionalExtensionCapability};

pub(super) fn advertised_capabilities() -> Vec<&'static str> {
    advertised_capabilities_for(!cfg!(target_os = "linux"))
}

pub(super) fn advertised_capabilities_for(contextual_apis: bool) -> Vec<&'static str> {
    types::EXTENSION_CAPABILITIES
        .iter()
        .copied()
        .chain(
            [
                OptionalExtensionCapability::Skills,
                OptionalExtensionCapability::Resources,
                OptionalExtensionCapability::RichToolResults,
                OptionalExtensionCapability::Models,
                OptionalExtensionCapability::Memory,
                OptionalExtensionCapability::Automations,
                OptionalExtensionCapability::Subagents,
                OptionalExtensionCapability::ToolInterception,
            ]
            .iter()
            .filter(|capability| {
                contextual_apis
                    || matches!(
                        capability,
                        OptionalExtensionCapability::Skills
                            | OptionalExtensionCapability::Resources
                            | OptionalExtensionCapability::RichToolResults
                    )
            })
            .map(OptionalExtensionCapability::as_str),
        )
        .collect()
}

pub(super) fn validate_negotiated_capabilities(values: &[String]) -> Result<(), String> {
    let advertised = advertised_capabilities()
        .into_iter()
        .collect::<BTreeSet<_>>();
    if values.is_empty()
        || values.len() > advertised.len()
        || values
            .iter()
            .any(|value| !advertised.contains(value.as_str()))
        || values.iter().collect::<BTreeSet<_>>().len() != values.len()
    {
        return Err(super::error_codes::HOST_INCOMPATIBLE.to_string());
    }
    Ok(())
}
