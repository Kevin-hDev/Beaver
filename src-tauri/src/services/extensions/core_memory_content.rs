use crate::services::agent_local::memory_types::{MemoryTopic, MAX_TOPIC_BYTES};

pub(super) fn for_create(
    content: &str,
    id: &str,
    scope: &str,
    session_id: &str,
    status: &str,
    source: &str,
) -> Result<String, ()> {
    let now = chrono::Utc::now().to_rfc3339();
    rewrite(
        content,
        [
            ("id", id),
            ("scope", scope),
            ("status", status),
            ("created_at", &now),
            ("updated_at", &now),
            ("source", source),
            ("session_id", session_id),
        ],
    )
}

pub(super) fn for_update(
    content: &str,
    current: &MemoryTopic,
    scope: &str,
) -> Result<String, ()> {
    let now = chrono::Utc::now().to_rfc3339();
    rewrite(
        content,
        [
            ("id", current.id.as_str()),
            ("scope", scope),
            ("status", current.status.as_str()),
            ("created_at", current.created_at.as_str()),
            ("updated_at", &now),
            ("source", current.source.as_str()),
            ("session_id", current.session_id.as_str()),
        ],
    )
}

fn rewrite(content: &str, replacements: [(&str, &str); 7]) -> Result<String, ()> {
    if content.is_empty() || content.len() > MAX_TOPIC_BYTES || content.contains('\0') {
        return Err(());
    }
    let mut output = String::with_capacity(content.len().saturating_add(128));
    let mut in_frontmatter = false;
    let mut closed = false;
    let mut seen = [false; 7];
    for (index, line) in content.lines().enumerate() {
        if line.trim() == "---" {
            if index == 0 {
                in_frontmatter = true;
            } else if in_frontmatter {
                in_frontmatter = false;
                closed = true;
            }
            output.push_str("---\n");
            continue;
        }
        if in_frontmatter {
            let key = line.split_once(':').map(|(key, _)| key.trim());
            if let Some(position) = replacements.iter().position(|(name, _)| Some(*name) == key) {
                if seen[position] {
                    return Err(());
                }
                seen[position] = true;
                output.push_str(replacements[position].0);
                output.push_str(": ");
                output.push_str(replacements[position].1);
                output.push('\n');
                continue;
            }
        }
        output.push_str(line);
        output.push('\n');
    }
    if !closed || seen.iter().any(|value| !value) || output.len() > MAX_TOPIC_BYTES {
        return Err(());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    #[test]
    fn controlled_fields_are_unique_and_owned_by_rust() {
        let input = "---\nid: old\nscope: global\ntype: preference\nstatus: stale\ntitle: T\nsummary: S\ncreated_at: old\nupdated_at: old\ntags: []\nsource: parent\nsession_id: old\n---\nBody";
        let output = super::for_create(input, "new", "project", "session", "confirmed", "user")
            .unwrap();
        assert!(output.contains("id: new\nscope: project"));
        assert!(output.contains("status: confirmed"));
        assert!(output.contains("session_id: session"));
        assert_eq!(output.lines().filter(|line| line.starts_with("id:")).count(), 1);
    }
}
