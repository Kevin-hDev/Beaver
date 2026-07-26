use super::memory_types::{MemoryTopic, MAX_TAGS, MAX_TOPIC_BYTES};
use chrono::{DateTime, Utc};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

const TYPES: &[&str] = &["preference", "feedback", "project", "reference"];
const STATUSES: &[&str] = &["confirmed", "inferred", "stale", "archived"];
const SOURCES: &[&str] = &["user", "parent", "extractor", "subagent-suggestion"];

static SECRET_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"(?i)-----BEGIN [A-Z ]*PRIVATE KEY-----",
        r"\bAKIA[0-9A-Z]{16}\b",
        r"\b(?:sk|pk)-[A-Za-z0-9_-]{20,}\b",
        r"\bgh[pousr]_[A-Za-z0-9]{20,}\b",
        r"\bxox[baprs]-[A-Za-z0-9-]{20,}\b",
        r"\beyJ[A-Za-z0-9_-]{5,}\.[A-Za-z0-9_-]{5,}\.[A-Za-z0-9_-]{5,}\b",
        r#"(?i)\b(?:api[_ -]?key|access[_ -]?token|password|secret)\s*[:=]\s*["']?[^\s"']{8,}"#,
    ]
    .into_iter()
    .map(|pattern| Regex::new(pattern).expect("valid memory secret regex"))
    .collect()
});

#[derive(Debug, Clone)]
pub struct ParsedTopic {
    pub topic: MemoryTopic,
}

pub fn parse(
    content: &str,
    file_path: &Path,
    expected_scope: &str,
) -> Result<ParsedTopic, String> {
    if content.is_empty() || content.len() > MAX_TOPIC_BYTES {
        return Err("Sujet mémoire invalide.".into());
    }
    if contains_secret(content) {
        return Err("Les données sensibles ne peuvent pas être mémorisées.".into());
    }
    let (frontmatter, body) = split_frontmatter(content)?;
    let fields = parse_fields(frontmatter)?;
    let id = required(&fields, "id", 64)?;
    let file_id = file_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Sujet mémoire invalide.".to_string())?;
    if uuid::Uuid::parse_str(&id).is_err() || id != file_id {
        return Err("Identifiant de sujet mémoire invalide.".into());
    }
    let scope = required(&fields, "scope", 16)?;
    if scope != expected_scope {
        return Err("Portée mémoire invalide.".into());
    }
    let memory_type = one_of(&fields, "type", TYPES)?;
    let status = one_of(&fields, "status", STATUSES)?;
    let source = one_of(&fields, "source", SOURCES)?;
    let title = required(&fields, "title", 120)?;
    let summary = required(&fields, "summary", 240)?;
    let created_at = timestamp(&fields, "created_at")?;
    let updated_at = timestamp(&fields, "updated_at")?;
    let session_id = required(&fields, "session_id", 64)?;
    if uuid::Uuid::parse_str(&session_id).is_err() {
        return Err("Provenance mémoire invalide.".into());
    }
    let tags = parse_tags(fields.get("tags").map(String::as_str).unwrap_or("[]"))?;
    if body.trim().is_empty() {
        return Err("Le contenu du sujet mémoire est vide.".into());
    }
    Ok(ParsedTopic {
        topic: MemoryTopic {
            id,
            title,
            summary,
            memory_type,
            status,
            tags,
            created_at,
            updated_at,
            source,
            session_id,
            path: file_path.to_string_lossy().into_owned(),
        },
    })
}

pub fn contains_secret(content: &str) -> bool {
    SECRET_PATTERNS.iter().any(|pattern| pattern.is_match(content))
}

fn split_frontmatter(content: &str) -> Result<(&str, &str), String> {
    let rest = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
        .ok_or_else(|| "En-tête du sujet mémoire manquant.".to_string())?;
    let marker = rest
        .find("\n---\n")
        .or_else(|| rest.find("\r\n---\r\n"))
        .ok_or_else(|| "En-tête du sujet mémoire incomplet.".to_string())?;
    let marker_len = if rest[marker..].starts_with("\r\n---\r\n") {
        7
    } else {
        5
    };
    Ok((&rest[..marker], &rest[marker + marker_len..]))
}

fn parse_fields(frontmatter: &str) -> Result<HashMap<String, String>, String> {
    let mut fields = HashMap::new();
    for line in frontmatter.lines() {
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| "En-tête du sujet mémoire invalide.".to_string())?;
        let key = key.trim();
        let value = value.trim().trim_matches(['"', '\'']);
        if key.is_empty() || value.chars().count() > 512 || fields.len() >= 16 {
            return Err("En-tête du sujet mémoire invalide.".into());
        }
        fields.insert(key.to_string(), value.to_string());
    }
    Ok(fields)
}

fn required(
    fields: &HashMap<String, String>,
    key: &str,
    max_chars: usize,
) -> Result<String, String> {
    let value = fields
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty() && value.chars().count() <= max_chars)
        .ok_or_else(|| "Champ du sujet mémoire invalide.".to_string())?;
    Ok(value.to_string())
}

fn one_of(
    fields: &HashMap<String, String>,
    key: &str,
    allowed: &[&str],
) -> Result<String, String> {
    let value = required(fields, key, 32)?;
    allowed
        .contains(&value.as_str())
        .then_some(value)
        .ok_or_else(|| {
            format!(
                "Valeur `{key}` invalide. Valeurs autorisées : {}.",
                allowed.join(", ")
            )
        })
}

fn timestamp(fields: &HashMap<String, String>, key: &str) -> Result<String, String> {
    let value = required(fields, key, 64)?;
    DateTime::parse_from_rfc3339(&value)
        .map(|date| date.with_timezone(&Utc).to_rfc3339())
        .map_err(|_| "Date du sujet mémoire invalide.".to_string())
}

fn parse_tags(value: &str) -> Result<Vec<String>, String> {
    let inner = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| "Tags mémoire invalides.".to_string())?;
    let tags = inner
        .split(',')
        .map(|tag| tag.trim().trim_matches(['"', '\'']).to_lowercase())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    if tags.len() > MAX_TAGS
        || tags
            .iter()
            .any(|tag| tag.chars().count() > 32 || !valid_tag(tag))
    {
        return Err("Tags mémoire invalides.".into());
    }
    Ok(tags)
}

fn valid_tag(tag: &str) -> bool {
    tag.chars()
        .all(|character| character.is_alphanumeric() || matches!(character, '-' | '_'))
}

#[cfg(test)]
#[path = "memory_format_tests.rs"]
mod tests;
