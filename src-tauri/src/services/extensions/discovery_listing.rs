use serde::Serialize;

use super::registry_index::IndexedPlugin;

#[derive(Serialize)]
pub(crate) struct ListedExtension {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tools: usize,
    pub skills: usize,
    pub resources: usize,
}

pub(crate) fn list(plugins: &[IndexedPlugin]) -> Result<Vec<ListedExtension>, ()> {
    if plugins.len() > super::discovery_contract::HOST_MAX_EXTENSIONS {
        return Err(());
    }
    let mut stable = plugins.to_vec();
    stable.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(stable
        .iter()
        .map(|plugin| ListedExtension {
            id: plugin.id.clone(),
            name: json_text(
                &plugin.name,
                super::discovery_contract::MAX_PROJECTED_EXTENSION_NAME_JSON_BYTES,
            ),
            description: json_text(
                plugin.description.as_deref().unwrap_or_default(),
                super::discovery_contract::MAX_PROJECTED_EXTENSION_DESCRIPTION_JSON_BYTES,
            ),
            tools: plugin.tools.len(),
            skills: plugin.skills.len(),
            resources: plugin.resources.len(),
        })
        .collect())
}

pub fn compact_catalog(plugins: &[IndexedPlugin]) -> Result<String, String> {
    if plugins.len() > super::discovery_contract::HOST_MAX_EXTENSIONS {
        return Err(super::error_codes::LISTING_UNAVAILABLE.to_string());
    }
    let entries = plugins
        .iter()
        .map(|plugin| CompactEntry {
            name: json_text(
                &plugin.name,
                super::discovery_contract::MAX_PROJECTED_EXTENSION_NAME_JSON_BYTES,
            ),
            id: &plugin.id,
        })
        .collect::<Vec<_>>();
    let serialized = serde_json::to_string(&entries)
        .map_err(|_| super::error_codes::LISTING_UNAVAILABLE.to_string())?;
    if serialized.len() > super::MAX_COMPACT_CATALOG_BYTES {
        return Err(super::error_codes::LISTING_UNAVAILABLE.to_string());
    }
    Ok(serialized)
}

#[derive(Serialize)]
struct CompactEntry<'a> {
    name: String,
    id: &'a str,
}

pub(crate) fn json_text(value: &str, maximum_bytes: usize) -> String {
    let mut output = String::new();
    for character in value.chars() {
        let mut candidate = output.clone();
        candidate.push(character);
        if serde_json::to_vec(&candidate).is_ok_and(|json| json.len() > maximum_bytes) {
            break;
        }
        output = candidate;
    }
    output
}

#[cfg(test)]
#[path = "discovery_listing_tests.rs"]
mod tests;
