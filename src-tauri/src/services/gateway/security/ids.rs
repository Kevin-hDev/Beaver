const MAX_ID_LEN: usize = 128;

pub fn validate_channel_id(id: &str) -> Result<(), String> {
    match id {
        "telegram" | "slack" | "discord" => Ok(()),
        _ => Err("canal invalide".to_string()),
    }
}

pub fn validate_account_id(id: &str) -> Result<(), String> {
    validate_chars(id, "identifiant de compte invalide")
}

pub fn validate_external_id(id: &str) -> Result<(), String> {
    validate_chars(id, "identifiant externe invalide")
}

fn validate_chars(id: &str, message: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > MAX_ID_LEN {
        return Err(message.to_string());
    }
    if !id.chars().all(is_allowed_id_char) {
        return Err(message.to_string());
    }
    Ok(())
}

fn is_allowed_id_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':' | '@' | '#')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_platform_ids() {
        assert!(validate_external_id("-100123456").is_ok());
        assert!(validate_external_id("123.456").is_ok());
        assert!(validate_external_id("C123:thread").is_ok());
    }

    #[test]
    fn rejects_bad_ids() {
        assert!(validate_account_id("").is_err());
        assert!(validate_account_id("../secret").is_err());
        assert!(validate_external_id(&"a".repeat(MAX_ID_LEN + 1)).is_err());
    }

    #[test]
    fn validates_channels() {
        assert!(validate_channel_id("telegram").is_ok());
        assert!(validate_channel_id("email").is_err());
    }
}
