use serde_json::Value;

const INVALID_MODELFILE: &str = "ollama-modelfile-invalid";

pub fn rewrite(content: &str, entries: &[(String, String)]) -> Result<String, String> {
    super::ollama_parameter_validation::validate_parameter_entries(entries)?;
    if content.trim().is_empty() || content.contains('\0') {
        return Err(INVALID_MODELFILE.into());
    }

    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let keep_final_line_ending = content.ends_with('\n');
    let mut lines = Vec::new();
    let mut insertion_index = None;
    let mut in_multiline = false;

    for raw_line in content.lines() {
        let line = raw_line.trim_end_matches('\r');
        if !in_multiline && is_parameter_directive(line) {
            insertion_index.get_or_insert(lines.len());
            continue;
        }
        lines.push(line.to_string());
        if triple_quote_count(line) % 2 == 1 {
            in_multiline = !in_multiline;
        }
    }

    let rendered = entries
        .iter()
        .map(|(key, value)| render_parameter(key.trim(), value.trim()))
        .collect::<Result<Vec<_>, _>>()?;
    let insert_at = insertion_index.unwrap_or_else(|| trailing_content_index(&lines));
    lines.splice(insert_at..insert_at, rendered);

    let mut output = lines.join(line_ending);
    if keep_final_line_ending || !entries.is_empty() {
        output.push_str(line_ending);
    }
    Ok(output)
}

fn is_parameter_directive(line: &str) -> bool {
    line.trim_start()
        .split_ascii_whitespace()
        .next()
        .is_some_and(|word| word.eq_ignore_ascii_case("PARAMETER"))
}

fn triple_quote_count(line: &str) -> usize {
    line.match_indices("\"\"\"").count()
}

fn trailing_content_index(lines: &[String]) -> usize {
    lines
        .iter()
        .rposition(|line| !line.trim().is_empty())
        .map_or(0, |index| index + 1)
}

fn render_parameter(key: &str, raw: &str) -> Result<String, String> {
    let value = parse_value(raw);
    let rendered = match value {
        Value::String(value) => quote_modelfile_text(&value),
        other => other.to_string(),
    };
    Ok(format!("PARAMETER {key} {rendered}"))
}

fn parse_value(raw: &str) -> Value {
    if let Some(value) = raw
        .strip_prefix("\"\"\"")
        .and_then(|value| value.strip_suffix("\"\"\""))
    {
        return Value::String(value.to_string());
    }
    if let Some(value) = raw
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        return Value::String(value.to_string());
    }
    super::modelfile_parser::parse_param_value(raw)
}

fn quote_modelfile_text(value: &str) -> String {
    if value.contains('\n') || value.starts_with(char::is_whitespace) || value.ends_with(char::is_whitespace) {
        if value.contains('"') {
            return format!("\"\"\"{value}\"\"\"");
        }
        return format!("\"{value}\"");
    }
    value.to_string()
}
