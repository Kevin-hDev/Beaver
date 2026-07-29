pub(super) const SLUG_MAX_CHARS: usize = 36;
const LABEL_SCAN_MAX_CHARS: usize = 256;

pub(super) fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut separator = false;
    let mut emitted = 0;
    for character in value.chars().take(LABEL_SCAN_MAX_CHARS) {
        if emitted >= SLUG_MAX_CHARS {
            break;
        }
        if character.is_ascii_alphanumeric() {
            if separator && !slug.is_empty() && emitted < SLUG_MAX_CHARS {
                slug.push('-');
                emitted += 1;
            }
            separator = false;
            if emitted < SLUG_MAX_CHARS {
                slug.push(character.to_ascii_lowercase());
                emitted += 1;
            }
        } else {
            separator = true;
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() || windows_reserved(slug) {
        "session".to_string()
    } else {
        slug.to_string()
    }
}

pub(super) fn session_suffix(session_id: &str) -> Result<String, String> {
    let suffix: String = session_id
        .chars()
        .filter(|character| character.is_ascii_hexdigit())
        .take(8)
        .collect();
    (suffix.len() == 8)
        .then_some(suffix)
        .ok_or_else(super::workspace_error)
}

pub(super) fn valid_date(value: &str) -> bool {
    value.len() == 10
        && value
            .chars()
            .enumerate()
            .all(|(index, character)| index == 4 || index == 7 || character.is_ascii_digit())
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
}

fn windows_reserved(value: &str) -> bool {
    matches!(
        value.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}
