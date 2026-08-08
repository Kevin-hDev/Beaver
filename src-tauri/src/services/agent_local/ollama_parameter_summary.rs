const MAX_PARAMETER_SUMMARY_BYTES: usize = 160 * 1024;
const MAX_PARAMETER_ENTRIES: usize = 128;
const MAX_PARAMETER_KEY_BYTES: usize = 64;
const MAX_PARAMETER_VALUE_BYTES: usize = 1024;
const INVALID_RESPONSE: &str = "ollama-invalid-response";

pub fn parse(summary: &str) -> Result<Vec<(String, String)>, String> {
    if summary.len() > MAX_PARAMETER_SUMMARY_BYTES || summary.contains('\0') {
        return Err(invalid_response());
    }

    let mut entries = Vec::new();
    for line in summary.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if entries.len() >= MAX_PARAMETER_ENTRIES {
            return Err(invalid_response());
        }
        let key_end = line
            .find(char::is_whitespace)
            .ok_or_else(invalid_response)?;
        let key = &line[..key_end];
        let raw_value = line[key_end..].trim_start();
        if !valid_key(key) || raw_value.is_empty() {
            return Err(invalid_response());
        }
        let value = if raw_value.starts_with('"') {
            decode_go_string(raw_value)?
        } else {
            raw_value.trim_end().to_string()
        };
        if value.len() > MAX_PARAMETER_VALUE_BYTES {
            return Err(invalid_response());
        }
        entries.push((key.to_string(), value));
    }
    Ok(entries)
}

fn decode_go_string(raw: &str) -> Result<String, String> {
    let mut chars = raw.char_indices().peekable();
    if chars.next().map(|(_, value)| value) != Some('"') {
        return Err(invalid_response());
    }
    let mut decoded = String::new();
    while let Some((index, character)) = chars.next() {
        match character {
            '"' => {
                if !raw[index + 1..].trim().is_empty() {
                    return Err(invalid_response());
                }
                return Ok(decoded);
            }
            '\\' => decode_escape(&mut chars, &mut decoded)?,
            value if value.is_control() => return Err(invalid_response()),
            value => decoded.push(value),
        }
    }
    Err(invalid_response())
}

fn decode_escape(
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    output: &mut String,
) -> Result<(), String> {
    let escaped = chars
        .next()
        .map(|(_, value)| value)
        .ok_or_else(invalid_response)?;
    match escaped {
        'a' => output.push('\u{7}'),
        'b' => output.push('\u{8}'),
        'f' => output.push('\u{c}'),
        'n' => output.push('\n'),
        'r' => output.push('\r'),
        't' => output.push('\t'),
        'v' => output.push('\u{b}'),
        '\\' => output.push('\\'),
        '"' => output.push('"'),
        'x' => output.push(read_scalar(chars, 2)?),
        'u' => output.push(read_scalar(chars, 4)?),
        'U' => output.push(read_scalar(chars, 8)?),
        first @ '0'..='7' => output.push(read_octal_scalar(chars, first)?),
        _ => return Err(invalid_response()),
    }
    Ok(())
}

fn read_scalar(
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    digits: usize,
) -> Result<char, String> {
    let mut value = 0_u32;
    for _ in 0..digits {
        let digit = chars
            .next()
            .and_then(|(_, character)| character.to_digit(16))
            .ok_or_else(invalid_response)?;
        value = value
            .checked_mul(16)
            .and_then(|n| n.checked_add(digit))
            .ok_or_else(invalid_response)?;
    }
    char::from_u32(value).ok_or_else(invalid_response)
}

fn read_octal_scalar(
    chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    first: char,
) -> Result<char, String> {
    let mut value = first.to_digit(8).ok_or_else(invalid_response)?;
    for _ in 0..2 {
        let digit = chars
            .next()
            .and_then(|(_, character)| character.to_digit(8))
            .ok_or_else(invalid_response)?;
        value = value * 8 + digit;
    }
    char::from_u32(value).ok_or_else(invalid_response)
}

fn valid_key(key: &str) -> bool {
    if key.is_empty() || key.len() > MAX_PARAMETER_KEY_BYTES {
        return false;
    }
    let mut chars = key.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn invalid_response() -> String {
    INVALID_RESPONSE.to_string()
}
