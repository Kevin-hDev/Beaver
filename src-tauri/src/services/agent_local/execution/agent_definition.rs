use super::types_tools::ToolResult;
use std::io::Read;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const MAX_AGENT_BYTES: u64 = 32 * 1024;
const MAX_NAME_CHARS: usize = 64;
const MAX_DESCRIPTION_CHARS: usize = 250;
const MAX_BODY_CHARS: usize = 12_000;

pub struct AgentDefinition {
    pub name: String,
    pub description: String,
    pub profile: String,
    pub body: String,
}

pub fn load(relative_path: &str, working_dir: &Path) -> Result<AgentDefinition, ToolResult> {
    let path = resolve_path(relative_path, working_dir)?;
    let metadata = std::fs::metadata(&path).map_err(|_| invalid_definition())?;
    if !metadata.is_file() || metadata.len() > MAX_AGENT_BYTES {
        return Err(invalid_definition());
    }
    let mut content = String::new();
    std::fs::File::open(&path)
        .and_then(|file| file.take(MAX_AGENT_BYTES).read_to_string(&mut content))
        .map_err(|_| invalid_definition())?;
    parse(&content)
}

fn resolve_path(relative_path: &str, working_dir: &Path) -> Result<PathBuf, ToolResult> {
    let requested = Path::new(relative_path);
    if relative_path.trim().is_empty() || requested.is_absolute() || relative_path.contains("..") {
        return Err(invalid_path());
    }
    let root = working_dir.canonicalize().map_err(|_| invalid_path())?;
    let path = root.join(requested).canonicalize().map_err(|_| invalid_path())?;
    if !path.starts_with(&root) {
        return Err(invalid_path());
    }
    Ok(path)
}

fn parse(content: &str) -> Result<AgentDefinition, ToolResult> {
    let trimmed = content.trim();
    let after_open = trimmed.strip_prefix("---\n").ok_or_else(invalid_definition)?;
    let close = after_open.find("\n---\n").ok_or_else(invalid_definition)?;
    let (frontmatter, body_with_marker) = after_open.split_at(close);
    let body = body_with_marker.trim_start_matches("\n---\n").trim();
    let mut name = None;
    let mut description = None;
    let mut profile = None;
    let mut seen = BTreeSet::new();
    for line in frontmatter.lines() {
        let (key, value) = line.split_once(':').ok_or_else(invalid_definition)?;
        if !seen.insert(key.trim()) {
            return Err(invalid_definition());
        }
        let value = strip_quotes(value.trim());
        match key.trim() {
            "name" => name = Some(value),
            "description" => description = Some(value),
            "profile" => profile = Some(value),
            _ => return Err(invalid_definition()),
        }
    }
    let definition = AgentDefinition {
        name: required_bounded(name, MAX_NAME_CHARS)?,
        description: required_bounded(description, MAX_DESCRIPTION_CHARS)?,
        profile: required_bounded(profile, 16)?,
        body: body.to_string(),
    };
    if !matches!(definition.profile.as_str(), "explorer" | "coder")
        || definition.body.is_empty()
        || definition.body.chars().count() > MAX_BODY_CHARS
    {
        return Err(invalid_definition());
    }
    Ok(definition)
}

fn required_bounded(value: Option<String>, max: usize) -> Result<String, ToolResult> {
    value
        .filter(|value| !value.trim().is_empty() && value.chars().count() <= max)
        .ok_or_else(invalid_definition)
}

fn strip_quotes(value: &str) -> String {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        return value[1..value.len() - 1].to_string();
    }
    value.to_string()
}

fn invalid_path() -> ToolResult {
    ToolResult::validation("agent_path_invalid", "Définition d'agent inaccessible.")
}

fn invalid_definition() -> ToolResult {
    ToolResult::validation("agent_definition_invalid", "Définition d'agent invalide.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_bounded_project_agent() {
        let temp = tempfile::TempDir::new().unwrap();
        let dir = temp.path().join(".beaver/agents");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("reviewer.md"),
            "---\nname: reviewer\ndescription: Reviews migrations.\nprofile: explorer\n---\n# Role\nYou inspect migrations.",
        )
        .unwrap();

        let agent = load(".beaver/agents/reviewer.md", temp.path()).unwrap();

        assert_eq!(agent.name, "reviewer");
        assert_eq!(agent.profile, "explorer");
        assert!(agent.body.contains("You inspect migrations."));
    }

    #[test]
    fn rejects_escape_and_unknown_fields() {
        let temp = tempfile::TempDir::new().unwrap();
        assert!(load("../agent.md", temp.path()).is_err());
        let path = temp.path().join("agent.md");
        std::fs::write(
            &path,
            "---\nname: a\ndescription: b\nprofile: coder\nmodel: x\n---\nBody",
        )
        .unwrap();
        assert!(load("agent.md", temp.path()).is_err());
        std::fs::write(
            &path,
            "---\nname: a\nname: b\ndescription: b\nprofile: coder\n---\nBody",
        )
        .unwrap();
        assert!(load("agent.md", temp.path()).is_err());
    }
}
