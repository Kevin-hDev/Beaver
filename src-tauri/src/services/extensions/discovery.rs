use super::discovery_text::{auto_query, clip_chars, normalize, terms};
use super::registry_index::IndexedTool;

pub const SEARCH_TOOL_NAME: &str = "search_extension_tools";
pub const MAX_SEARCH_QUERY_CHARS: usize = 512;
pub const MAX_SEARCH_RESULTS: usize = 12;
pub const MAX_SELECTED_TOOLS: usize = 64;
const MAX_AUTO_QUERY_CHARS: usize = 4096;
const MIN_RELEVANCE_SCORE: u32 = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolMatch {
    pub extension_id: String,
    pub extension_name: String,
    pub tool_name: String,
    pub description: String,
    pub score: u32,
}

pub fn search(query: &str, limit: usize) -> Vec<ToolMatch> {
    ranked(query, limit, MAX_SEARCH_QUERY_CHARS)
}

fn ranked(query: &str, limit: usize, max_query_chars: usize) -> Vec<ToolMatch> {
    let query = clip_chars(query, max_query_chars);
    let mut matches = super::registry_index::indexed_tools()
        .into_iter()
        .filter_map(|tool| ranked_match(&query, tool))
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.tool_name.cmp(&right.tool_name))
    });
    matches.truncate(limit.min(MAX_SEARCH_RESULTS));
    matches
}

pub fn select_plugin_tools(query: &str, limit: usize) -> Vec<String> {
    let query = auto_query(query);
    if query.is_empty() {
        return Vec::new();
    }
    select_plugins(
        ranked(&query, MAX_SEARCH_RESULTS, MAX_AUTO_QUERY_CHARS),
        limit,
        MIN_RELEVANCE_SCORE,
    )
}

pub fn discover_plugin_tools(query: &str, limit: usize) -> Vec<String> {
    select_plugins(search(query, MAX_SEARCH_RESULTS), limit, 1)
}

fn select_plugins(matches: Vec<ToolMatch>, limit: usize, minimum_score: u32) -> Vec<String> {
    let mut extension_ids = Vec::new();
    for item in matches.iter().filter(|item| item.score >= minimum_score) {
        if !extension_ids.contains(&item.extension_id) {
            extension_ids.push(item.extension_id.clone());
        }
    }
    select_complete_plugins(
        &super::registry_index::indexed_tools(),
        &extension_ids,
        limit.min(MAX_SELECTED_TOOLS),
    )
}

fn select_complete_plugins(
    indexed: &[IndexedTool],
    extension_ids: &[String],
    limit: usize,
) -> Vec<String> {
    let mut selected = Vec::new();
    for extension_id in extension_ids {
        let plugin_tools = indexed
            .iter()
            .filter(|item| &item.extension_id == extension_id)
            .map(|item| item.tool.name.clone())
            .collect::<Vec<_>>();
        if selected.len() + plugin_tools.len() <= limit {
            selected.extend(plugin_tools);
        }
    }
    selected
}

fn ranked_match(query: &str, indexed: IndexedTool) -> Option<ToolMatch> {
    let terms = terms(query);
    if terms.is_empty() {
        return None;
    }
    let identity = normalize(&format!(
        "{} {} {}",
        indexed.extension_id, indexed.extension_name, indexed.tool.name
    ));
    let descriptions = normalize(&format!(
        "{} {}",
        indexed.extension_description, indexed.tool.description
    ));
    let parameters = normalize(
        &indexed
            .tool
            .parameters
            .get("properties")
            .and_then(serde_json::Value::as_object)
            .map(|properties| properties.keys().cloned().collect::<Vec<_>>().join(" "))
            .unwrap_or_default(),
    );
    let score = terms
        .iter()
        .map(|term| {
            field_score(&identity, term, 8)
                + field_score(&descriptions, term, 5)
                + field_score(&parameters, term, 2)
        })
        .sum();
    (score > 0).then_some(ToolMatch {
        extension_id: indexed.extension_id,
        extension_name: indexed.extension_name,
        tool_name: indexed.tool.name,
        description: indexed.tool.description,
        score,
    })
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

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
