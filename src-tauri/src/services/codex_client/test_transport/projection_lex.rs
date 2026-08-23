use super::ScanError;

pub(super) fn whitespace(body: &[u8], cursor: &mut usize) {
    while matches!(body.get(*cursor), Some(b' ' | b'\n' | b'\r' | b'\t')) {
        *cursor += 1;
    }
}

pub(super) fn byte(body: &[u8], cursor: &mut usize, expected: u8) -> Result<(), ScanError> {
    if body.get(*cursor) != Some(&expected) {
        return Err(ScanError);
    }
    *cursor += 1;
    Ok(())
}

pub(super) fn optional_byte(body: &[u8], cursor: &mut usize, expected: u8) -> bool {
    if body.get(*cursor) != Some(&expected) {
        return false;
    }
    *cursor += 1;
    true
}

pub(super) fn literal(body: &[u8], cursor: &mut usize, expected: &[u8]) -> Result<(), ScanError> {
    let end = cursor.checked_add(expected.len()).ok_or(ScanError)?;
    if body.get(*cursor..end) != Some(expected) {
        return Err(ScanError);
    }
    *cursor = end;
    Ok(())
}

pub(super) fn number(body: &[u8], cursor: &mut usize) -> Result<(), ScanError> {
    optional_byte(body, cursor, b'-');
    match body.get(*cursor).copied() {
        Some(b'0') => {
            *cursor += 1;
            if body.get(*cursor).is_some_and(u8::is_ascii_digit) {
                return Err(ScanError);
            }
        }
        Some(b'1'..=b'9') => digits(body, cursor),
        _ => return Err(ScanError),
    }
    if optional_byte(body, cursor, b'.') {
        required_digits(body, cursor)?;
    }
    if matches!(body.get(*cursor), Some(b'e' | b'E')) {
        *cursor += 1;
        if matches!(body.get(*cursor), Some(b'+' | b'-')) {
            *cursor += 1;
        }
        required_digits(body, cursor)?;
    }
    Ok(())
}

fn required_digits(body: &[u8], cursor: &mut usize) -> Result<(), ScanError> {
    if !body.get(*cursor).is_some_and(u8::is_ascii_digit) {
        return Err(ScanError);
    }
    digits(body, cursor);
    Ok(())
}

fn digits(body: &[u8], cursor: &mut usize) {
    while body.get(*cursor).is_some_and(u8::is_ascii_digit) {
        *cursor += 1;
    }
}
