use serde_json::Value;
use sha2::{Digest, Sha256};

pub(crate) const MAX_PROVIDER_TOOL_NAME: usize = 64;
const HASH_HEX_CHARS: usize = 32;
const MAX_ALIAS_CONTEXT_NAMES: usize = 512;

pub(crate) struct ToolNameMap {
    names: Vec<String>,
}

impl ToolNameMap {
    pub(crate) fn new(tools: &[Value]) -> Self {
        let mut names = tools
            .iter()
            .filter_map(tool_name)
            .map(str::to_string)
            .take(MAX_ALIAS_CONTEXT_NAMES)
            .collect::<Vec<_>>();
        let remaining = MAX_ALIAS_CONTEXT_NAMES.saturating_sub(names.len());
        names.extend(
            crate::services::extensions::dynamic_tool_names()
                .into_iter()
                .take(remaining),
        );
        names.sort();
        names.dedup();
        Self { names }
    }

    pub(crate) fn wire_name(&self, name: &str) -> String {
        let alias = wire_name(name);
        let collision = self
            .names
            .iter()
            .filter(|candidate| candidate.as_str() != name)
            .any(|candidate| wire_name(candidate) == alias);
        if collision && name != alias {
            collision_alias(name, &alias)
        } else {
            alias
        }
    }

    pub(crate) fn wire_name_for_provider(&self, provider_id: &str, name: &str) -> String {
        let alias = provider_alias(provider_id, name);
        let collision = self
            .names
            .iter()
            .filter(|candidate| candidate.as_str() != name)
            .any(|candidate| provider_alias(provider_id, candidate) == alias);
        if collision && name != alias {
            collision_alias(name, &alias)
        } else {
            alias
        }
    }

    pub(crate) fn restore(&self, name: &str, tools: &[Value]) -> String {
        tools
            .iter()
            .filter_map(tool_name)
            .find(|candidate| self.wire_name(candidate) == name)
            .unwrap_or(name)
            .to_string()
    }

    pub(crate) fn restore_for_provider(
        &self,
        provider_id: &str,
        name: &str,
        tools: &[Value],
    ) -> String {
        tools
            .iter()
            .filter_map(tool_name)
            .find(|candidate| self.wire_name_for_provider(provider_id, candidate) == name)
            .unwrap_or(name)
            .to_string()
    }
}

fn provider_alias(provider_id: &str, name: &str) -> String {
    if provider_id == "qwen" && name == "search" {
        "beaver_search".to_string()
    } else {
        wire_name(name)
    }
}

pub(crate) fn wire_name(name: &str) -> String {
    bounded_alias(name, &readable_alias(name))
}

#[cfg(test)]
pub(crate) fn wire_name_with_tools(name: &str, tools: &[Value]) -> String {
    ToolNameMap::new(tools).wire_name(name)
}

pub(crate) fn restore_tool_name(name: &str, tools: &[Value]) -> String {
    ToolNameMap::new(tools).restore(name, tools)
}

pub(crate) fn restore_tool_name_for_provider(
    provider_id: &str,
    name: &str,
    tools: &[Value],
) -> String {
    ToolNameMap::new(tools).restore_for_provider(provider_id, name, tools)
}

fn readable_alias(name: &str) -> String {
    let mut alias = String::with_capacity(name.len());
    let mut previous_separator = false;
    for character in name.chars() {
        let mapped = if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
            character
        } else {
            '_'
        };
        if mapped == '_' && previous_separator {
            continue;
        }
        previous_separator = mapped == '_';
        alias.push(mapped);
    }
    if alias.is_empty() {
        alias.push_str("tool");
    } else if !alias
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
    {
        alias.insert_str(0, "tool_");
    }
    alias
}

fn bounded_alias(name: &str, alias: &str) -> String {
    if alias.len() <= MAX_PROVIDER_TOOL_NAME {
        alias.to_string()
    } else {
        hashed_alias(name, alias)
    }
}

fn collision_alias(name: &str, alias: &str) -> String {
    hashed_alias(&format!("collision:{name}"), alias)
}

fn hashed_alias(name: &str, stem: &str) -> String {
    let digest = Sha256::digest(name.as_bytes());
    let suffix = hex::encode(&digest[..HASH_HEX_CHARS / 2]);
    let stem_chars = MAX_PROVIDER_TOOL_NAME - suffix.len() - 1;
    format!(
        "{}_{suffix}",
        stem.chars().take(stem_chars).collect::<String>()
    )
}

fn tool_name(tool: &Value) -> Option<&str> {
    tool.pointer("/function/name").and_then(Value::as_str)
}

#[cfg(test)]
pub(crate) fn has_provider_name_shape(name: &str) -> bool {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    name.len() <= MAX_PROVIDER_TOOL_NAME
        && (first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
}
