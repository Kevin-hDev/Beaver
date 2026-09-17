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
    // Count the JSON quotes and each scalar's escape once, without serializing prefixes.
    let mut bytes = 2;
    for character in value.chars() {
        let escaped = match character {
            '"' | '\\' | '\n' | '\r' | '\t' | '\u{0008}' | '\u{000c}' => 2,
            '\u{0000}'..='\u{001f}' => 6,
            _ => character.len_utf8(),
        };
        if bytes + escaped > maximum_bytes {
            break;
        }
        bytes += escaped;
        output.push(character);
    }
    output
}

#[cfg(test)]
#[path = "discovery_listing_tests.rs"]
mod tests;
