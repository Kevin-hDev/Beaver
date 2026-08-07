const INVALID_MODELFILE: &str = "ollama-modelfile-invalid";

#[derive(Clone, Copy)]
enum QuoteDelimiter {
    Single,
    Triple,
}

pub fn rewrite(content: &str, entries: &[(String, String)]) -> Result<String, String> {
    super::ollama_parameter_validation::validate_parameter_entries(entries)?;
    if content.trim().is_empty() || content.contains('\0') {
        return Err(INVALID_MODELFILE.into());
    }

    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
    let keep_final_line_ending = content.ends_with('\n');
    let mut lines = Vec::new();
    let mut insertion_index = None;
    let mut multiline = None;

    for raw_line in content.lines() {
        let line = raw_line.trim_end_matches('\r');
        if let Some(delimiter) = multiline {
            lines.push(line.to_string());
            if closes_multiline(line, delimiter) {
                multiline = None;
            }
            continue;
        }

        let opening = opening_delimiter(line);
        if is_parameter_directive(line) && opening.is_none() {
            insertion_index.get_or_insert(lines.len());
            continue;
        }
        lines.push(line.to_string());
        multiline = opening;
    }

    let rendered = entries
        .iter()
        .map(|(key, value)| render_parameter(key.trim(), value.trim()))
        .collect::<Vec<_>>();
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

fn opening_delimiter(line: &str) -> Option<QuoteDelimiter> {
    let value = directive_value(line)?.trim();
    if value.starts_with("\"\"\"") {
        return (!(value.len() >= 6 && value.ends_with("\"\"\""))).then_some(QuoteDelimiter::Triple);
    }
    if value.starts_with('"') {
        return (!(value.len() >= 2 && value.ends_with('"'))).then_some(QuoteDelimiter::Single);
    }
    None
}

fn directive_value(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if line.starts_with('#') {
        return None;
    }
    let (directive, rest) = line.split_once(char::is_whitespace)?;
    let rest = rest.trim_start();
    if directive.eq_ignore_ascii_case("PARAMETER")
        || directive.eq_ignore_ascii_case("MESSAGE")
    {
        return rest
            .split_once(char::is_whitespace)
            .map(|(_, value)| value.trim_start());
    }
    Some(rest)
}

fn closes_multiline(line: &str, delimiter: QuoteDelimiter) -> bool {
    let line = line.trim_end();
    match delimiter {
        QuoteDelimiter::Single => line.ends_with('"'),
        QuoteDelimiter::Triple => line.ends_with("\"\"\""),
    }
}

fn trailing_content_index(lines: &[String]) -> usize {
    lines
        .iter()
        .rposition(|line| !line.trim().is_empty())
        .map_or(0, |index| index + 1)
}

fn render_parameter(key: &str, value: &str) -> String {
    format!("PARAMETER {key} {}", quote_modelfile_text(value))
}

fn quote_modelfile_text(value: &str) -> String {
    if value.contains('"') {
        return format!("\"\"\"{value}\"\"\"");
    }
    value.to_string()
}
