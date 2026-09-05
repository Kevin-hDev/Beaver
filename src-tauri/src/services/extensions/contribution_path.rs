pub fn validate(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.chars().count() > super::types::MAX_PATH_CHARS
        || value.starts_with('/')
        || value.contains('\\')
        || value.contains(':')
        || value.chars().any(|character| character.is_control())
        || !value.split('/').all(valid_component)
    {
        return Err(());
    }
    Ok(())
}

fn valid_component(component: &str) -> bool {
    !component.is_empty() && component != "." && component != ".." && !dos_reserved_name(component)
}

fn dos_reserved_name(component: &str) -> bool {
    // Windows accepts these names even with an extension or trailing dots/spaces.
    let normalized = component.trim_end_matches([' ', '.']);
    let base = normalized
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || base
            .strip_prefix("COM")
            .or_else(|| base.strip_prefix("LPT"))
            .is_some_and(|number| {
                matches!(number, "¹" | "²" | "³")
                    || (number.len() == 1 && matches!(number.as_bytes(), [b'1'..=b'9']))
            })
}
