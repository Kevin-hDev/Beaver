use serde::{Deserialize, Serialize};

use super::types::ExtensionResourceType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionResource {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub resource_type: ExtensionResourceType,
    pub path: String,
}
