use std::ops::Range;

const INVALID_MODELFILE: &str = "ollama-modelfile-invalid";
const MAX_MODELFILE_BYTES: usize = 4 * 1024 * 1024;
const MAX_PARAMETER_CANDIDATES: usize = 512;

pub fn rewrite(
    content: &str,
    current_entries: &[(String, String)],
    new_entries: &[(String, String)],
) -> Result<String, String> {
    super::ollama_parameter_validation::validate_parameter_entries(new_entries)?;
    validate_content(content)?;

    let normalized = content.replace("\r\n", "\n");
    if normalized.contains('\r') {
        return Err(invalid_modelfile());
    }

    let rendered_new = new_entries
        .iter()
        .map(|(key, value)| {
            let value = super::ollama_parameter_validation::value_for_render(key, value);
            render_source_parameter(key.trim(), value)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let updated = if current_entries.is_empty() {
        append_parameters(&normalized, &rendered_new)
    } else {
        let range = locate_parameter_block(&normalized, current_entries)?;
        let replacement = if rendered_new.is_empty() {
            String::new()
        } else {
            format!("{rendered_new}\n")
        };
        let mut output = normalized;
        output.replace_range(range, &replacement);
        output
    };

    Ok(updated)
}

fn locate_parameter_block(
    content: &str,
    entries: &[(String, String)],
) -> Result<Range<usize>, String> {
    let mut rendered = Vec::<(String, usize)>::new();
    for (key, value) in entries {
        let line = render_normalized_parameter(key, value);
        if let Some((_, count)) = rendered.iter_mut().find(|(known, _)| known == &line) {
            *count += 1;
        } else {
            rendered.push((line, 1));
        }
    }

    let mut candidates = Vec::new();
    let mut candidate_count = 0;
    for (start, _) in content.match_indices("PARAMETER ") {
        if start > 0 && content.as_bytes()[start - 1] != b'\n' {
            continue;
        }
        candidate_count += 1;
        if candidate_count > MAX_PARAMETER_CANDIDATES {
            return Err(invalid_modelfile());
        }
        if let Some(end) = match_complete_block(content, start, &rendered, entries.len()) {
            candidates.push(start..end);
            if candidates.len() > 1 {
                return Err(invalid_modelfile());
            }
        }
    }
    candidates.pop().ok_or_else(invalid_modelfile)
}

fn match_complete_block(
    content: &str,
    start: usize,
    expected: &[(String, usize)],
    total: usize,
) -> Option<usize> {
    let mut remaining = expected.to_vec();
    let mut cursor = start;
    for _ in 0..total {
        let match_index = remaining.iter().position(|(rendered, count)| {
            *count > 0 && matches_complete_entry(content, cursor, rendered)
        })?;
        cursor += remaining[match_index].0.len();
        remaining[match_index].1 -= 1;
        if cursor < content.len() {
            cursor += 1;
        }
    }
    if content[cursor..].starts_with("PARAMETER ") {
        return None;
    }
    Some(cursor)
}

fn matches_complete_entry(content: &str, start: usize, rendered: &str) -> bool {
    let Some(rest) = content.get(start..) else {
        return false;
    };
    if !rest.starts_with(rendered) {
        return false;
    }
    let end = start + rendered.len();
    end == content.len() || content.as_bytes().get(end) == Some(&b'\n')
}

fn append_parameters(content: &str, rendered: &str) -> String {
    if rendered.is_empty() {
        return content.to_string();
    }
    let mut output = content.to_string();
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(rendered);
    output.push('\n');
    output
}

fn render_normalized_parameter(key: &str, value: &str) -> String {
    format!("PARAMETER {} {}", key.trim(), quote_normalized_text(value))
}

fn render_source_parameter(key: &str, value: &str) -> String {
    format!("PARAMETER {} {}", key.trim(), quote_source_text(value))
}

fn quote_normalized_text(value: &str) -> String {
    let needs_quotes = value.contains('\n') || value.starts_with(' ') || value.ends_with(' ');
    if !needs_quotes {
        return value.to_string();
    }
    if value.contains('"') {
        return format!("\"\"\"{value}\"\"\"");
    }
    format!("\"{value}\"")
}

fn quote_source_text(value: &str) -> String {
    if value.starts_with('"') {
        return format!("\"\"\"{value}\"\"\"");
    }
    let edge_whitespace = value
        .chars()
        .next()
        .is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace);
    if !value.contains('\n') && !edge_whitespace {
        return value.to_string();
    }
    if value.contains('"') {
        return format!("\"\"\"{value}\"\"\"");
    }
    format!("\"{value}\"")
}

fn validate_content(content: &str) -> Result<(), String> {
    if content.trim().is_empty() || content.len() > MAX_MODELFILE_BYTES || content.contains('\0') {
        return Err(invalid_modelfile());
    }
    Ok(())
}

fn invalid_modelfile() -> String {
    INVALID_MODELFILE.to_string()
}
