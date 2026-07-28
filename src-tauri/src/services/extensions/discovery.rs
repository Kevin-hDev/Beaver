use std::collections::HashSet;

use super::registry_index::IndexedPlugin;

pub const SEARCH_TOOL_NAME: &str = "search_extension_tools";
pub const MAX_SEARCH_QUERY_CHARS: usize = 512;
pub const MAX_SEARCH_RESULTS: usize = 12;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginMatch {
    pub extension_id: String,
    pub extension_name: String,
    pub score: u32,
}

pub fn search(query: &str, limit: usize) -> Vec<PluginMatch> {
    let query = clip_chars(query, MAX_SEARCH_QUERY_CHARS);
    let mut matches = super::registry_index::indexed_plugins()
        .into_iter()
        .filter_map(|plugin| ranked_match(&query, plugin))
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.extension_name.cmp(&right.extension_name))
            .then_with(|| left.extension_id.cmp(&right.extension_id))
    });
    matches.truncate(limit.min(MAX_SEARCH_RESULTS));
    matches
}

fn ranked_match(query: &str, plugin: IndexedPlugin) -> Option<PluginMatch> {
    let terms = terms(query);
    if terms.is_empty() {
        return None;
    }
    let identity = normalize(&format!("{} {}", plugin.id, plugin.name));
    let descriptions = normalize(&format!(
        "{} {}",
        plugin.description.as_deref().unwrap_or_default(),
        plugin
            .tools
            .iter()
            .map(|tool| tool.description.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    ));
    let tools = normalize(
        &plugin
            .tools
            .iter()
            .map(|tool| tool.name.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    );
    let score = terms
        .iter()
        .map(|term| {
            field_score(&identity, term, 8)
                + field_score(&descriptions, term, 5)
                + field_score(&tools, term, 3)
        })
        .sum();
    (score > 0).then_some(PluginMatch {
        extension_id: plugin.id,
        extension_name: plugin.name,
        score,
    })
}

fn terms(value: &str) -> Vec<String> {
    normalize(value)
        .split_whitespace()
        .filter(|term| term.chars().count() >= 2)
        .map(str::to_string)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .map(fold_latin)
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect()
}

fn fold_latin(character: char) -> char {
    match character {
        'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
        'ç' => 'c',
        'è' | 'é' | 'ê' | 'ë' => 'e',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'ñ' => 'n',
        'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
        'ù' | 'ú' | 'û' | 'ü' => 'u',
        'ý' | 'ÿ' => 'y',
        other => other,
    }
}

fn field_score(field: &str, term: &str, weight: u32) -> u32 {
    if field.split_whitespace().any(|word| word == term) {
        weight
    } else if term.chars().count() >= 4 && field.contains(term) {
        weight.saturating_sub(1)
    } else {
        0
    }
}

fn clip_chars(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
