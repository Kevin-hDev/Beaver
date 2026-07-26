pub fn archive(content: &str) -> Result<String, String> {
    update_frontmatter(content, &chrono::Utc::now().to_rfc3339())
}

fn update_frontmatter(content: &str, updated_at: &str) -> Result<String, String> {
    let mut output = String::with_capacity(content.len().saturating_add(32));
    let mut delimiters = 0usize;
    let mut status_updates = 0usize;
    let mut time_updates = 0usize;

    for line in content.lines() {
        if line.trim() == "---" {
            delimiters = delimiters.saturating_add(1);
            output.push_str("---\n");
            continue;
        }
        if delimiters == 1 && line.starts_with("status:") {
            output.push_str("status: archived\n");
            status_updates = status_updates.saturating_add(1);
        } else if delimiters == 1 && line.starts_with("updated_at:") {
            output.push_str(&format!("updated_at: {updated_at}\n"));
            time_updates = time_updates.saturating_add(1);
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    if delimiters < 2 || status_updates != 1 || time_updates != 1 {
        return Err("Sujet mémoire invalide.".into());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_updates_only_frontmatter_fields() {
        let input = "---\nstatus: confirmed\nupdated_at: old\n---\nstatus: body";
        let output = update_frontmatter(input, "2026-07-24T20:00:00Z").unwrap();

        assert!(output.contains("status: archived"));
        assert!(output.contains("updated_at: 2026-07-24T20:00:00Z"));
        assert!(output.ends_with("status: body\n"));
    }
}
