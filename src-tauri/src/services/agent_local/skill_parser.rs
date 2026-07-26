use std::io::Read;
use std::path::Path;

const MAX_METADATA_BYTES: u64 = 32 * 1024;

pub fn read_skill_metadata(
    path: &Path,
    fallback_name: &str,
    max_file_bytes: u64,
) -> Option<(String, String)> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > max_file_bytes {
        return None;
    }
    let file = std::fs::File::open(path).ok()?;
    let mut bytes = Vec::with_capacity(MAX_METADATA_BYTES as usize);
    file.take(MAX_METADATA_BYTES)
        .read_to_end(&mut bytes)
        .ok()?;
    let prefix = String::from_utf8_lossy(&bytes);
    let (name, description, _) = parse_skill_content(&prefix, fallback_name);
    Some((name, description))
}

/// Parsing YAML frontmatter pour les fichiers skill.md / SKILL.md.
/// Retourne (name, description, body) — body = contenu sans frontmatter.
pub fn parse_skill_content(content: &str, fallback_name: &str) -> (String, String, String) {
    let trimmed = content.trim();
    if !trimmed.starts_with("---") {
        let desc = trimmed
            .lines()
            .find(|l| !l.is_empty() && !l.starts_with('#'))
            .unwrap_or("")
            .chars()
            .take(120)
            .collect::<String>();
        return (fallback_name.to_string(), desc, trimmed.to_string());
    }

    let after_open = &trimmed[3..];
    let close_pos = after_open.find("\n---");
    let (yaml_block, body) = match close_pos {
        Some(pos) => {
            let yaml = &after_open[..pos];
            let rest = &after_open[pos + 4..];
            (yaml.trim(), rest.trim())
        }
        None => {
            return (
                fallback_name.to_string(),
                String::new(),
                trimmed.to_string(),
            )
        }
    };

    let mut name = String::new();
    let mut description = String::new();
    for line in yaml_block.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("name:") {
            name = strip_yaml_quotes(val.trim());
        } else if let Some(val) = line.strip_prefix("description:") {
            description = strip_yaml_quotes(val.trim());
        }
    }

    if name.is_empty() {
        name = fallback_name.to_string();
    }
    if description.is_empty() {
        description = body
            .lines()
            .find(|l| !l.is_empty() && !l.starts_with('#'))
            .unwrap_or("")
            .chars()
            .take(120)
            .collect();
    }

    (name, description, body.to_string())
}

fn strip_yaml_quotes(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frontmatter_full() {
        let content =
            "---\nname: Mon Skill\ndescription: Un skill de test\n---\n# Contenu\nHello world";
        let (name, desc, body) = parse_skill_content(content, "fallback");
        assert_eq!(name, "Mon Skill");
        assert_eq!(desc, "Un skill de test");
        assert_eq!(body, "# Contenu\nHello world");
    }

    #[test]
    fn parse_frontmatter_no_name_uses_fallback() {
        let content = "---\ndescription: Desc only\n---\nBody here";
        let (name, desc, body) = parse_skill_content(content, "my-skill");
        assert_eq!(name, "my-skill");
        assert_eq!(desc, "Desc only");
        assert_eq!(body, "Body here");
    }

    #[test]
    fn parse_no_frontmatter() {
        let content = "# Just markdown\nNo frontmatter here";
        let (name, desc, body) = parse_skill_content(content, "raw-skill");
        assert_eq!(name, "raw-skill");
        assert_eq!(desc, "No frontmatter here");
        assert_eq!(body, "# Just markdown\nNo frontmatter here");
    }

    #[test]
    fn parse_empty_content() {
        let (name, desc, body) = parse_skill_content("", "empty");
        assert_eq!(name, "empty");
        assert_eq!(desc, "");
        assert_eq!(body, "");
    }

    #[test]
    fn parse_frontmatter_strips_quotes() {
        let content = "---\nname: \"Quoted Name\"\ndescription: 'Single quoted'\n---\nBody";
        let (name, desc, body) = parse_skill_content(content, "fallback");
        assert_eq!(name, "Quoted Name");
        assert_eq!(desc, "Single quoted");
        assert_eq!(body, "Body");
    }

    #[test]
    fn metadata_reader_uses_only_bounded_prefix() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("SKILL.md");
        let mut content = b"---\nname: bounded\ndescription: Prefix only\n---\n".to_vec();
        content.extend(vec![b'x'; MAX_METADATA_BYTES as usize]);
        content.push(0xff);
        std::fs::write(&path, content).unwrap();

        let metadata = read_skill_metadata(&path, "fallback", 256 * 1024).unwrap();

        assert_eq!(metadata.0, "bounded");
        assert_eq!(metadata.1, "Prefix only");
    }
}
