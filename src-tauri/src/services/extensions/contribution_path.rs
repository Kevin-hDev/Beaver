pub fn validate(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.chars().count() > super::types::MAX_PATH_CHARS
        || value.starts_with('/')
        || value.contains('\\')
        || value.contains(':')
        || value.chars().any(|character| character.is_control())
        || !value
            .split('/')
            .all(|component| !component.is_empty() && component != "." && component != "..")
    {
        return Err(());
    }
    Ok(())
}
