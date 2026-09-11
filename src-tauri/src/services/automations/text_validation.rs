use super::AutomationError;

pub(crate) fn validate_single_line_text(value: &str, max: usize) -> Result<(), AutomationError> {
    validate_text(value, max, false)
}

pub(crate) fn validate_multiline_text(value: &str, max: usize) -> Result<(), AutomationError> {
    validate_text(value, max, true)
}

pub(crate) fn validate_optional_single_line_text(
    value: Option<&str>,
    max: usize,
) -> Result<(), AutomationError> {
    validate_optional_text(value, max, false)
}

pub(crate) fn validate_optional_multiline_text(
    value: Option<&str>,
    max: usize,
) -> Result<(), AutomationError> {
    validate_optional_text(value, max, true)
}

fn validate_text(value: &str, max: usize, multiline: bool) -> Result<(), AutomationError> {
    if value.trim().is_empty()
        || value.chars().count() > max
        || value
            .chars()
            .any(|character| disallowed_control(character, multiline))
    {
        return Err(AutomationError::InvalidInput);
    }
    Ok(())
}

fn validate_optional_text(
    value: Option<&str>,
    max: usize,
    multiline: bool,
) -> Result<(), AutomationError> {
    if value.is_some_and(|value| {
        value.chars().count() > max
            || value
                .chars()
                .any(|character| disallowed_control(character, multiline))
    }) {
        return Err(AutomationError::InvalidInput);
    }
    Ok(())
}

fn disallowed_control(character: char, multiline: bool) -> bool {
    character.is_control() && !(multiline && matches!(character, '\n' | '\r' | '\t'))
}
