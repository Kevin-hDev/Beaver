use serde::Serialize;

use super::registry_index::IndexedPlugin;

#[derive(Serialize)]
pub(crate) struct InspectedExtension {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: InspectionStatus,
    pub tools: Vec<Contribution>,
    pub skills: Vec<Contribution>,
    pub resources: Vec<Contribution>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InspectionStatus {
    Unknown,
    Inactive,
    Unapproved,
    AlreadyAvailable,
    Loaded,
    NoTools,
    LimitedByProvider,
}

#[derive(Serialize)]
pub(crate) struct Contribution {
    pub id: String,
    pub name: String,
    pub summary: String,
}

pub(crate) fn inspect(plugin: &IndexedPlugin, status: InspectionStatus) -> InspectedExtension {
    InspectedExtension {
        id: plugin.id.clone(),
        name: super::discovery_listing::json_text(
            &plugin.name,
            super::discovery_contract::MAX_PROJECTED_EXTENSION_NAME_JSON_BYTES,
        ),
        description: super::discovery_listing::json_text(
            plugin.description.as_deref().unwrap_or_default(),
            super::discovery_contract::MAX_PROJECTED_EXTENSION_DESCRIPTION_JSON_BYTES,
        ),
        status,
        tools: plugin
            .tools
            .iter()
            .map(|tool| Contribution {
                id: tool.name.clone(),
                name: contribution_name(&tool.name),
                summary: contribution_summary(&tool.description),
            })
            .collect(),
        skills: plugin
            .skills
            .iter()
            .map(|skill| Contribution {
                id: skill.id.clone(),
                name: contribution_name(&skill.name),
                summary: contribution_summary(&skill.description),
            })
            .collect(),
        resources: plugin
            .resources
            .iter()
            .map(|resource| Contribution {
                id: resource.id.clone(),
                name: contribution_name(&resource.name),
                summary: contribution_summary(&resource.description),
            })
            .collect(),
    }
}

fn contribution_name(value: &str) -> String {
    super::discovery_listing::json_text(
        value,
        super::discovery_contract::MAX_PROJECTED_CONTRIBUTION_NAME_JSON_BYTES,
    )
}

fn contribution_summary(value: &str) -> String {
    super::discovery_listing::json_text(
        value,
        super::discovery_contract::MAX_PROJECTED_CONTRIBUTION_SUMMARY_JSON_BYTES,
    )
}

#[cfg(test)]
#[path = "discovery_inspection_tests.rs"]
mod tests;
