const MAX_WORKSPACE_CHARS: usize = 64;

pub(crate) fn validate(value: &str) -> Result<(), &'static str> {
    let len = value.chars().count();
    if !(1..=MAX_WORKSPACE_CHARS).contains(&len)
        || value.starts_with('-')
        || value.ends_with('-')
        || !value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        return Err("provider_configuration_invalid");
    }
    Ok(())
}
