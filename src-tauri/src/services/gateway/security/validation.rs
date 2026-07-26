const MAX_BYTES: usize = 32_768;
const MAX_CHARS: usize = 8_000;

pub struct ValidationResult {
    pub valid: bool,
    pub reason: Option<String>,
}

impl ValidationResult {
    fn ok() -> Self {
        Self {
            valid: true,
            reason: None,
        }
    }
    fn reject(reason: &str) -> Self {
        Self {
            valid: false,
            reason: Some(reason.into()),
        }
    }
}

pub fn validate_message(content: &str) -> ValidationResult {
    if content.len() > MAX_BYTES {
        return ValidationResult::reject("message trop volumineux");
    }
    let char_count = content.chars().count();
    if char_count > MAX_CHARS {
        return ValidationResult::reject("message trop long");
    }
    if has_forbidden_control_chars(content) {
        return ValidationResult::reject("caractères interdits");
    }
    ValidationResult::ok()
}

fn has_forbidden_control_chars(s: &str) -> bool {
    s.chars()
        .any(|c| c.is_control() && c != '\n' && c != '\t' && c != '\r')
}

#[cfg(test)]
fn utf16_len(s: &str) -> usize {
    s.chars().map(|c| c.len_utf16()).sum()
}

pub fn split_utf16(text: &str, max_units: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_len = 0usize;

    for c in text.chars() {
        let c_len = c.len_utf16();
        if current_len + c_len > max_units && !current.is_empty() {
            chunks.push(std::mem::take(&mut current));
            current_len = 0;
        }
        current.push(c);
        current_len += c_len;
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_message() {
        assert!(validate_message("Hello world").valid);
    }

    #[test]
    fn rejects_oversized_bytes() {
        let big = "a".repeat(MAX_BYTES + 1);
        assert!(!validate_message(&big).valid);
    }

    #[test]
    fn rejects_too_many_chars() {
        let long = "é".repeat(MAX_CHARS + 1);
        assert!(!validate_message(&long).valid);
    }

    #[test]
    fn rejects_control_chars() {
        assert!(!validate_message("hello\x00world").valid);
        assert!(!validate_message("hello\x07world").valid);
    }

    #[test]
    fn allows_newline_and_tab() {
        assert!(validate_message("hello\nworld\ttab").valid);
    }

    #[test]
    fn utf16_len_ascii() {
        assert_eq!(utf16_len("hello"), 5);
    }

    #[test]
    fn utf16_len_emoji() {
        assert_eq!(utf16_len("😀"), 2);
        assert_eq!(utf16_len("a😀b"), 4);
    }

    #[test]
    fn split_utf16_respects_limit() {
        let text = "abc😀def";
        let chunks = split_utf16(text, 4);
        for chunk in &chunks {
            assert!(utf16_len(chunk) <= 4);
        }
        let rejoined: String = chunks.into_iter().collect();
        assert_eq!(rejoined, text);
    }
}
